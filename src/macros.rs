#[macro_export]
macro_rules! declare_client_traits {
    ($name:ty) => {
        impl std::ops::Deref for $name
        where
            $name: ::airmash_client::ImplClient,
        {
            type Target = ::airmash_client::ImplClient;

            fn deref(&self) -> &ImplClient {
                unsafe { std::mem::transmute(self) }
            }
        }

        impl std::ops::DerefMut for $name
        where
            $name: ::airmash_client::Client,
        {
            fn deref_mut(&mut self) -> &mut ImplClient {
                unsafe { std::mem::transmute(self) }
            }
        }
    };
}
