//! TODO

use core::ops::Range;
use std::{env, fs, path::Path};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{File, Ident, LitInt};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let u8_trait = Trait::new(IntType::U8);
    let u16_trait = Trait::new(IntType::U16);
    let u32_trait = Trait::new(IntType::U32);
    let u64_trait = Trait::new(IntType::U64);
    let u128_trait = Trait::new(IntType::U128);
    let usize_trait = Trait::new(IntType::Usize);

    let file: File = syn::parse_quote! {
        #u8_trait
        #u16_trait
        #u32_trait
        #u64_trait
        #u128_trait
        #usize_trait
    };

    let output = format!("// @generated\n\n{}", prettyplease::unparse(&file));
    fs::write(dest_path, output).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

struct Trait {
    int_type: IntType,
    sets: Vec<ImplementationSet>,
}

impl Trait {
    #[must_use]
    fn new(int_type: IntType) -> Self {
        let mut sets = vec![ImplementationSet::new(
            TokenStream::new(),
            int_type,
            int_type.range_exponents(),
        )];

        if int_type == IntType::Usize {
            usize_correction(&mut sets);
        }

        Self { int_type, sets }
    }
}

fn usize_correction(sets: &mut Vec<ImplementationSet>) {
    let bit32_set = ImplementationSet::new(
        quote! {
            #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
            #[cfg_attr(docsrs, doc(cfg(any(target_pointer_width = "32", target_pointer_width = "64"))))]
        },
        IntType::Usize,
        16..32,
    );

    let bit64_set = ImplementationSet::new(
        quote! {
            #[cfg(target_pointer_width = "64")]
            #[cfg_attr(docsrs, doc(cfg(target_pointer_width = "64")))]
        },
        IntType::Usize,
        32..64,
    );

    sets.push(bit32_set);
    sets.push(bit64_set);
}

impl ToTokens for Trait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Trait { int_type, sets } = self;

        let trait_ident = int_type.trait_ident();
        let int_ty = int_type.ty();
        let impls = sets.iter().flat_map(ImplementationSet::implementations);

        let docs = vec![
            format!(" A marker trait for [`{int_ty}`] values that are a power of two."),
            String::from(" "),
            String::from(" # Examples"),
            String::from(" "),
            String::from(" You can use the trait as a bound in a constant generic context."),
            String::from(" ```"),
            format!(" use const_power_of_two::{trait_ident};"),
            String::from(" "),
            format!(" trait MyTrait<const N: {int_ty}>"),
            String::from(" where"),
            String::from("     // NOTE: This is how you use the trait."),
            format!("     {int_ty}: {trait_ident}<N>,"),
            String::from(" {"),
            String::from(" }"),
            String::from(" ```"),
            String::from(" "),
            String::from(
                " If you use something that is not a power of two, you will get a compile-time",
            ),
            String::from(" error."),
            String::from(" "),
            String::from(" ```compile_fail"),
            format!(" use const_power_of_two::{trait_ident};"),
            String::from(" "),
            String::from(" struct Test;"),
            String::from(" "),
            format!(" trait MyTrait<const N: {int_ty}>"),
            String::from(" where"),
            format!("     {int_ty}: {trait_ident}<N>,"),
            String::from(" {"),
            String::from(" }"),
            String::from(" "),
            format!(" // ERROR: the trait bound `{int_ty}: {trait_ident}<0>` is not satisfied"),
            String::from(" impl MyTrait<0> for Test {}"),
        ];

        tokens.append_all(quote! {
            #(#[doc = #docs])*
            pub trait #trait_ident<const N: #int_ty> {}
            #(#impls)*
        });
    }
}

/// Generates the `PowerOfTwo` trait for unsigned integer types.
struct ImplementationSet {
    attrs: TokenStream,
    int_type: IntType,
    exponents: Range<u32>,
}

impl ImplementationSet {
    #[must_use]
    fn new(attrs: TokenStream, int_type: IntType, exponents: Range<u32>) -> Self {
        Self {
            attrs,
            int_type,
            exponents,
        }
    }

    fn implementations(&self) -> impl Iterator<Item = Implementation> + '_ {
        self.exponents
            .clone()
            .map(|exponent| Implementation::new(&self.attrs, self.int_type, exponent))
    }
}

struct Implementation<'a> {
    attrs: &'a TokenStream,
    int_type: IntType,
    value: Literal,
}

impl<'a> Implementation<'a> {
    #[must_use]
    fn new(attrs: &'a TokenStream, int_type: IntType, exponent: u32) -> Self {
        let value = int_type.power_of_two(exponent);

        Self {
            attrs,
            int_type,
            value,
        }
    }
}

impl ToTokens for Implementation<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Implementation {
            attrs,
            int_type,
            value,
        } = self;

        let trait_ident = int_type.trait_ident();

        tokens.append_all(quote! {
            #attrs
            impl #trait_ident<#value> for #int_type {}
        });
    }
}

/// An integer literal that is printed without any suffix.
#[derive(Clone, Copy)]
struct Literal(u128);

impl Literal {
    /// Creates a new integer literal.
    #[must_use]
    fn new<T>(value: T) -> Self
    where
        T: Into<u128>,
    {
        Self(value.into())
    }
}

impl ToTokens for Literal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Literal(value) = *self;
        let input = value.to_string();

        let string = if input.len() > 3 {
            let mut output = Vec::<u8>::with_capacity(input.len() + (input.len() / 3) + 1);
            let bytes = input.as_bytes();

            let first_segment_length = bytes.len() % 3;

            if first_segment_length != 0 {
                output.extend_from_slice(&bytes[..first_segment_length]);
                output.push(b'_');
            }

            let mut index = first_segment_length;

            while index < bytes.len() {
                let segment = &bytes[index..index + 3];
                output.extend_from_slice(segment);
                index += 3;

                if index != bytes.len() {
                    output.push(b'_');
                }
            }

            debug_assert_eq!(index, bytes.len());

            String::from_utf8(output).unwrap()
        } else {
            input
        };

        let literal = LitInt::new(&string, Span::call_site());
        literal.to_tokens(tokens);
    }
}

/// An unsigned integer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntType {
    /// An 8-bit unsigned integer.
    U8,
    /// A 16-bit unsigned integer.
    U16,
    /// A 32-bit unsigned integer.
    U32,
    /// A 64-bit unsigned integer.
    U64,
    /// A 128-bit unsigned integer.
    U128,
    /// An architecture-dependent unsigned integer.
    Usize,
}

impl IntType {
    #[must_use]
    fn trait_ident(self) -> Ident {
        let string = match self {
            IntType::U8 => "PowerOfTwoU8",
            IntType::U16 => "PowerOfTwoU16",
            IntType::U32 => "PowerOfTwoU32",
            IntType::U64 => "PowerOfTwoU64",
            IntType::U128 => "PowerOfTwoU128",
            IntType::Usize => "PowerOfTwoUsize",
        };

        Ident::new(string, Span::call_site())
    }

    #[must_use]
    fn range_exponents(self) -> Range<u32> {
        match self {
            IntType::U8 => 0u32..8u32,
            // NOTE: The range for `usize` is architecture-dependent. This is
            // the minimum range that is supported by all architectures.
            IntType::U16 | IntType::Usize => 0u32..16u32,
            IntType::U32 => 0u32..32u32,
            IntType::U64 => 0u32..64u32,
            IntType::U128 => 0u32..128u32,
        }
    }

    #[must_use]
    fn power_of_two(self, exponent: u32) -> Literal {
        match self {
            IntType::U8 => Literal::new(2u8.pow(exponent)),
            IntType::U16 => Literal::new(2u16.pow(exponent)),
            IntType::U32 => Literal::new(2u32.pow(exponent)),
            IntType::U64 | IntType::Usize => Literal::new(2u64.pow(exponent)),
            IntType::U128 => Literal::new(2u128.pow(exponent)),
        }
    }

    #[must_use]
    fn ty(self) -> Ident {
        let string = match self {
            IntType::U8 => "u8",
            IntType::U16 => "u16",
            IntType::U32 => "u32",
            IntType::U64 => "u64",
            IntType::U128 => "u128",
            IntType::Usize => "usize",
        };

        Ident::new(string, Span::call_site())
    }
}

impl ToTokens for IntType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty().to_tokens(tokens);
    }
}
