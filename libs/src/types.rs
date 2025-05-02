pub type KonError = Box<dyn std::error::Error + Send + Sync>;
pub type KonResult<T> = Result<T, KonError>;
pub type PoiseCtx<'a> = poise::Context<'a, (), KonError>;
pub type PoiseFwCtx<'a> = poise::FrameworkContext<'a, (), KonError>;
