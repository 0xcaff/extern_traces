use anyhow::format_err;

pub trait BuildBase: Sized {
    type Builder: Initialize + Finalize<Self>;
}

pub trait Build<EntryType>: BuildBase {
    type Builder: Initialize + Finalize<Self> + Builder<EntryType> + Clone;
}

pub trait Builder<EntryType> {
    fn update(&mut self, entry: &EntryType) -> Option<()>;
}

pub trait Initialize {
    fn new() -> Self;
}

impl<T> Initialize for Option<T> {
    fn new() -> Self {
        None
    }
}

pub trait Finalize<TargetType> {
    fn finalize(self) -> Result<TargetType, anyhow::Error>;
}

impl<T> Finalize<T> for Option<T> {
    fn finalize(self) -> Result<T, anyhow::Error> {
        Ok(self.ok_or_else(|| format_err!("value missing!"))?)
    }
}

impl<T> Finalize<T> for T {
    fn finalize(self) -> Result<T, anyhow::Error> {
        Ok(self)
    }
}

pub trait Marker {}

impl Marker for u32 {}

impl<T: Marker> BuildBase for T {
    type Builder = Option<T>;
}

impl<T> BuildBase for Option<T> {
    type Builder = Option<T>;
}

impl<EntryType, T: Build<EntryType>> Build<EntryType> for Option<T> {
    type Builder = OptionBuilder<EntryType, T>;
}

pub struct OptionBuilder<EntryType, T: Build<EntryType>> {
    inner: <T as Build<EntryType>>::Builder,
    has_accepted_input: bool,
}

impl<EntryType, T: Build<EntryType>> Clone for OptionBuilder<EntryType, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            has_accepted_input: self.has_accepted_input.clone(),
        }
    }
}

impl<EntryType, T: Build<EntryType>> Initialize for OptionBuilder<EntryType, T> {
    fn new() -> Self {
        Self {
            inner: <T as Build<EntryType>>::Builder::new(),
            has_accepted_input: false,
        }
    }
}

impl<EntryType, T: Build<EntryType>> Builder<EntryType> for OptionBuilder<EntryType, T> {
    fn update(&mut self, entry: &EntryType) -> Option<()> {
        if let Some(()) = self.inner.update(entry) {
            self.has_accepted_input = true;

            return Some(());
        }

        return None;
    }
}

impl<EntryType, T: Build<EntryType>> Finalize<Option<T>> for OptionBuilder<EntryType, T> {
    fn finalize(self) -> Result<Option<T>, anyhow::Error> {
        if self.has_accepted_input {
            Ok(Some(self.inner.finalize()?))
        } else {
            Ok(None)
        }
    }
}
