//! Simple derivation of #[derive(HeapSize)]

extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate synstructure;

#[cfg(not(test))]
decl_derive!([HeapSize, attributes(heap_ignore)] => heap_size_derive);

fn heap_size_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
	let match_body = s.each(|binding| {
		let ignore = binding.ast().attrs.iter().any(|attr| match attr.parse_meta().unwrap() {
			syn::Meta::Path(ref path) | syn::Meta::List(syn::MetaList { ref path, .. })
				if path.is_ident("heap_ignore") =>
			{
				panic!(
					"#[heap_ignore] should have an explanation, \
					 e.g. #[heap_ignore = \"because reasons\"]"
				);
			}
			syn::Meta::NameValue(syn::MetaNameValue { ref path, .. }) if path.is_ident("heap_ignore") => true,
			_ => false,
		});
		if ignore {
			None
		} else if let syn::Type::Array(..) = binding.ast().ty {
			Some(quote! {
				for item in #binding.iter() {
					sum += sp_memory::HeapSize::size_of(item, ops);
				}
			})
		} else {
			Some(quote! {
				sum += sp_memory::HeapSize::size_of(#binding, ops);
			})
		}
	});

	let ast = s.ast();
	let name = &ast.ident;
	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
	let mut where_clause = where_clause.unwrap_or(&parse_quote!(where)).clone();
	for param in ast.generics.type_params() {
		let ident = &param.ident;
		where_clause.predicates.push(parse_quote!(#ident: sp_memory::HeapSize));
	}

	let tokens = quote! {
		impl #impl_generics sp_memory::HeapSize for #name #ty_generics #where_clause {
			#[inline]
			#[allow(unused_variables, unused_mut, unreachable_code)]
			fn size_of(&self, ops: &mut sp_memory::MallocSizeOfOps) -> usize {
				let mut sum = 0;
				match *self {
					#match_body
				}
				sum
			}
		}
	};

	tokens
}
