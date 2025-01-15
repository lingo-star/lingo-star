//! A "GlueGun CLI" is a Rust crate that creates the glue between the Rust code and
//! some other language. Most GlueGun CLI crates can use the Clap structs defined
//! in this file.

use std::path::PathBuf;

use accessors_rs::Accessors;
use serde::Deserialize;

use crate::{codegen::LibraryCrate, idl::Idl};

/// Trait implemented by gluegun helper applications.
/// Your `main` function should invoke [`run`][].
/// By convention, types that implement this trait should be named `GlueGunX` where `X` is the name of your helper.
pub trait GlueGunHelper {
    /// Returns the helper name that users provide to invoke this, e.g., for `gluegun-java`, returns `"java"`.
    fn name(&self) -> String;

    /// Generate a helper crate `dest_crate` given the `idl`
    fn generate(self, cx: &mut GenerateCx) -> anyhow::Result<()>;
}

/// The "main" function for a gluegun helper. Defines standard argument parsing.
pub fn run(helper: impl GlueGunHelper) -> anyhow::Result<()> {
    // cargo-gluegun will invoke us with `gg` as argument and a JSON doc on stdin.
    let mut args = std::env::args();
    let Some(arg0) = args.next() else {
        anyhow::bail!("expected to be invoked by `cargo gluegun`");
    };
    if arg0 != "gg" {
        anyhow::bail!("expected to be invoked by `cargo gluegun`");
    }

    // Parse the input from stdin
    let stdin = std::io::stdin();
    let input: GlueGunInput = serde_json::from_reader(stdin.lock())?;

    // Invoke the user's code
    let mut cx = GenerateCx {
        idl: input.idl,
        dest_crate: input.dest_crate,
    };
    helper.generate(&mut cx)
}

/// These are the subcommands executed by our system.
/// Your extension should be able to respond to them.
#[derive(Deserialize)]
struct GlueGunInput {
    idl: Idl,
    dest_crate: GlueGunDestinationCrate,
}

/// Context provided to the [`GlueGunHelper::generate`][] implementation.
#[derive(Accessors)]
#[accessors(get)]
pub struct GenerateCx {
    /// The IDL from the source crate
    idl: Idl,

    /// Informaton about the destination crate
    dest_crate: GlueGunDestinationCrate,
}

impl GenerateCx {
    /// Create a [`LibraryCrate`][] instance.
    pub fn create_library_crate(&mut self) -> LibraryCrate {
        LibraryCrate::from_args(&self.dest_crate)
    }
}

impl AsRef<GlueGunDestinationCrate> for GlueGunDestinationCrate {
    fn as_ref(&self) -> &GlueGunDestinationCrate {
        self
    }
}

/// The arguments that identify where the crate should be generated.
/// You don't normally need to inspect the fields of this struct,
/// instead just invoke [`LibraryCrate::from_args`](`crate::codegen::LibraryCrate::from_args`).
#[derive(Deserialize, Debug)]
#[non_exhaustive]
pub struct GlueGunDestinationCrate {
    /// Path at which to create the crate
    pub path: PathBuf,

    /// Name to give the crate; if `None`, then just let `cargo` pick a name.
    pub crate_name: Option<String>,
}
