#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return spl_token::solana_program::entrypoint::ProgramResult::Err($err);
        }
    };
}
