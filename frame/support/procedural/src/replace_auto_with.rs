use proc_macro2::{TokenStream, TokenTree, Group};
use syn::spanned::Spanned;
use std::iter::once;

pub fn replace_auto_with(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let def = syn::parse_macro_input!(input as ReplaceAutoWithDef);
	let replace_in_span = def.replace_in.span();

	match replace_in_stream(&mut Some(def.replace_with), def.replace_in) {
		Ok(stream) => stream.into(),
		Err(_) => {
			syn::Error::new(replace_in_span, "cannot find `auto` ident in given token stream")
				.to_compile_error().into()
		},
	}
}
struct ReplaceAutoWithDef {
	replace_with: TokenStream,
	replace_in: TokenStream,
}

impl syn::parse::Parse for ReplaceAutoWithDef {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let replace_with;
		let _replace_with_bracket: syn::token::Brace = syn::braced!(replace_with in input);
		let replace_with: TokenStream = replace_with.parse()?;
		Ok(Self {
			replace_with,
			replace_in: input.parse()?,
		})
	}
}

// Replace the first found `auto` ident by content of `with`. `with` must be some (Option is used
// for internal simplification).
fn replace_in_stream(
	with: &mut Option<TokenStream>,
	stream: TokenStream
) -> Result<TokenStream, ()> {
	assert!(with.is_some(), "`with` must be some, Option is used because `with` is used only once");

	let mut stream = stream.into_iter();
	let mut builded_replaced = TokenStream::new();

	loop {
		match stream.next() {
			Some(TokenTree::Group(group)) => {
				let stream = group.stream();
				match replace_in_stream(with, stream) {
					Ok(stream) => {
						builded_replaced.extend(once(TokenTree::Group(Group::new(group.delimiter(), stream))));
						break;
					}
					Err(_) => {
						builded_replaced.extend(once(TokenTree::Group(group)));
					}
				}
			}
			Some(TokenTree::Ident(ident)) if ident == "auto" => {
				builded_replaced.extend(once(with.take().expect("with is used to replace only once")));
				break;
			}
			Some(other) => builded_replaced.extend(once(other)),
			None => return Err(())
		}
	}

	builded_replaced.extend(stream);

	Ok(builded_replaced)
}

