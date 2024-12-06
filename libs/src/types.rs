use {
  super::KonData,
  std::error::Error
};

pub type KonError = Box<dyn Error + Send + Sync>;
pub type KonResult<T> = Result<T, KonError>;
pub type PoiseCtx<'a> = poise::Context<'a, KonData, KonError>;
pub type PoiseFwCtx<'a> = poise::FrameworkContext<'a, KonData, KonError>;
