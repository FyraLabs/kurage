#[macro_export]
macro_rules! generate_page {
    ($page:ident $({$($model:tt)+})? $(as $modelname:ident)?:
        $(
        init$([$($local_ref:ident)+])?($root:ident, $initsender:ident, $initmodel:ident, $initwidgets:ident) $initblock:block
        )?
        update($self:ident, $message:ident, $sender:ident) {
            $( $msg:ident$(($($param:ident: $paramtype:ty),+$(,)?))? => $msghdl:expr ),*$(,)?
        }
        => {$( $out:pat ),*}
        $($viewtt:tt)*
    ) => { $crate::paste::paste! {
        kurage_page_pre!();
        $crate::generate_component!(
            [<$page Page>]$({$($model)+})? $(as $modelname)?:
            $(init$([$($local_ref)+])?($root, $initsender, $initmodel, $initwidgets) $initblock)?
            update($self, $message, $sender) {
            Nav(action: NavAction) => $sender.output(Self::Output::Nav(action)).unwrap(),
            $( $msg$(($($param: $paramtype),+))? => $msghdl),*
            } => {Nav(NavAction), $($out),*}

            #[root]
            kurage_page_hdl_view!($($viewtt)*)
        );
    }};
}

#[rustfmt::skip]
#[macro_export]
macro_rules! gen_macros {
    () => {
        #[allow(unused_macros)]
        mod kurage_generated_macros {
            macro_rules! kurage_page_pre {
                () => {
                    use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
                };
            }
            pub(crate) use {kurage_page_pre};
        }
    };
}
pub use gen_macros;
