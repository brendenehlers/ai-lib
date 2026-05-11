#[macro_export]
macro_rules! define_model {
    (
        name = $name:ident,
        provider = $provider:path,
        model_name = $model_name:literal,
        capabilities = [$($cap:ident),*],
    ) => {
        pub struct $name {
            provider: $provider,
        }

        impl $name {
            pub fn new(provider: $provider) -> Self {
                $name { provider }
            }
        }

        impl $crate::capabilities::model::Model for $name {
            fn model_name(&self) -> &'static str {
                $model_name
            }
        }

        $($crate::__impl_capability!($name, $provider, $cap);)*
    };
}

#[macro_export]
macro_rules! __impl_capability {
    ($name:ident, $provider:path, ChatModel) => {
        impl $crate::capabilities::model::ChatModel for $name {
            fn generate_text(
                &self,
                request: $crate::capabilities::domain::GenerateTextRequest,
            ) -> impl ::core::future::Future<
                Output = $crate::errors::AiLibResult<
                    $crate::capabilities::domain::GenerateTextResponse,
                >,
            > + Send {
                use $crate::capabilities::provider::ChatProvider;
                self.provider.generate_text(request)
            }
        }
    };
    ($name:ident, $provider:path, EmbeddingModel) => {};
}
