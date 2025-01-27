/// Parses the following syntax:
///
/// ```no_run
/// generate_generator! { name =>
///    gtk::Label { ... },
///    Idk { whatelse: true },
/// }
/// ```
struct GenerateGeneratorSyn {
    name: syn::Ident,
    views: proc_macro2::TokenStream,
}

impl syn::parse::Parse for GenerateGeneratorSyn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![=>]>()?;
        let views = input.parse()?;
        Ok(Self { name, views })
    }
}

fn recurse_replace_kurage_inner(
    ts: proc_macro2::TokenStream,
) -> impl Iterator<Item = proc_macro2::TokenTree> {
    ts.into_iter().flat_map(|tt| match tt {
        proc_macro2::TokenTree::Ident(i) if i == "KURAGE_INNER" => {
            Box::new(quote::quote! { $($viewtt)* }.into_iter())
                as Box<dyn Iterator<Item = proc_macro2::TokenTree>>
        }
        proc_macro2::TokenTree::Group(group) => Box::new(std::iter::once(
            proc_macro2::TokenTree::Group(proc_macro2::Group::new(
                group.delimiter(),
                recurse_replace_kurage_inner(group.stream()).collect(),
            )),
        )),
        other => Box::new(std::iter::once(other)),
    })
}

#[proc_macro]
pub fn generate_generator(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let GenerateGeneratorSyn { name, views } =
        syn::parse_macro_input!(input as GenerateGeneratorSyn);
    let views: proc_macro2::TokenStream = recurse_replace_kurage_inner(views).collect();
    quote::quote! {
        macro_rules! #name {
            ($page:ident $({$($model:tt)+})? $(as $modelname:ident)?:
                $(
                init$([$($local_ref:ident)+])?($root:ident, $initsender:ident, $initmodel:ident, $initwidgets:ident) $initblock:block
                )?
                update($self:ident, $message:ident, $sender:ident) {
                    $( $msg:ident$(($($param:ident: $paramtype:ty),+$(,)?))? => $msghdl:expr ),*$(,)?
                }
                => {$( $out:pat ),*}
                $($viewtt:tt)*
            ) => { ::paste::paste! {
                kurage_page_pre!();
                kurage::generate_component!(
                    [<$page Page>]$({$($model)+})? $(as $modelname)?:
                    $(init$([$($local_ref)+])?($root, $initsender, $initmodel, $initwidgets) $initblock)?
                    update($self, $message, $sender) {
                    Nav(action: NavAction) => $sender.output(Self::Output::Nav(action)).unwrap(),
                    $( $msg$(($($param: $paramtype),+))? => $msghdl),*
                    } => {Nav(NavAction), $($out),*}

                    #views
                );
            }};
        }
    }.into()
}
