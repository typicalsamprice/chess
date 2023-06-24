#[cfg(feature = "pext")]
macro_rules! pext {
    ($a:expr, $b:expr) => {
        bitintr::Pext($a, $b)
    }
}

#[cfg(not(feature = "pext"))]
macro_rules! pext { ($a:expr, $b:expr) => { 0 }; }

pub(crate) use pext;
