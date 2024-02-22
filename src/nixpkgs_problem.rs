use crate::structure;
use crate::utils::PACKAGE_NIX_FILENAME;
use indoc::writedoc;
use relative_path::RelativePath;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Any problem that can occur when checking Nixpkgs
pub enum NixpkgsProblem {
    ShardNonDir {
        relative_shard_path: PathBuf,
    },
    InvalidShardName {
        relative_shard_path: PathBuf,
        shard_name: String,
    },
    PackageNonDir {
        relative_package_dir: PathBuf,
    },
    CaseSensitiveDuplicate {
        relative_shard_path: PathBuf,
        first: OsString,
        second: OsString,
    },
    InvalidPackageName {
        relative_package_dir: PathBuf,
        package_name: String,
    },
    IncorrectShard {
        relative_package_dir: PathBuf,
        correct_relative_package_dir: PathBuf,
    },
    PackageNixNonExistent {
        relative_package_dir: PathBuf,
    },
    PackageNixDir {
        relative_package_dir: PathBuf,
    },
    UndefinedAttr {
        relative_package_file: PathBuf,
        package_name: String,
    },
    WrongCallPackage {
        relative_package_file: PathBuf,
        package_name: String,
    },
    WrongCallPackagePath {
        package_name: String,
        file: PathBuf,
        line: usize,
        column: usize,
        actual_path: PathBuf,
        expected_path: PathBuf,
    },
    NonSyntacticCallPackage {
        package_name: String,
        file: PathBuf,
        line: usize,
        column: usize,
        definition: String,
    },
    NonDerivation {
        relative_package_file: PathBuf,
        package_name: String,
    },
    OutsideSymlink {
        relative_package_dir: PathBuf,
        subpath: PathBuf,
    },
    UnresolvableSymlink {
        relative_package_dir: PathBuf,
        subpath: PathBuf,
        io_error: io::Error,
    },
    PathInterpolation {
        relative_package_dir: PathBuf,
        subpath: PathBuf,
        line: usize,
        text: String,
    },
    SearchPath {
        relative_package_dir: PathBuf,
        subpath: PathBuf,
        line: usize,
        text: String,
    },
    OutsidePathReference {
        relative_package_dir: PathBuf,
        subpath: PathBuf,
        line: usize,
        text: String,
    },
    UnresolvablePathReference {
        relative_package_dir: PathBuf,
        subpath: PathBuf,
        line: usize,
        text: String,
        io_error: io::Error,
    },
    MovedOutOfByName {
        package_name: String,
        call_package_path: Option<PathBuf>,
        empty_arg: bool,
    },
    NewPackageNotUsingByName {
        package_name: String,
        call_package_path: Option<PathBuf>,
        empty_arg: bool,
    },
    InternalCallPackageUsed {
        attr_name: String,
    },
    CannotDetermineAttributeLocation {
        attr_name: String,
    },
}

impl fmt::Display for NixpkgsProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NixpkgsProblem::ShardNonDir { relative_shard_path } =>
                write!(
                    f,
                    "{}: This is a file, but it should be a directory.",
                    relative_shard_path.display(),
                ),
            NixpkgsProblem::InvalidShardName { relative_shard_path, shard_name } =>
                write!(
                    f,
                    "{}: Invalid directory name \"{shard_name}\", must be at most 2 ASCII characters consisting of a-z, 0-9, \"-\" or \"_\".",
                    relative_shard_path.display()
                ),
            NixpkgsProblem::PackageNonDir { relative_package_dir } =>
                write!(
                    f,
                    "{}: This path is a file, but it should be a directory.",
                    relative_package_dir.display(),
                ),
            NixpkgsProblem::CaseSensitiveDuplicate { relative_shard_path, first, second } =>
                write!(
                    f,
                    "{}: Duplicate case-sensitive package directories {first:?} and {second:?}.",
                    relative_shard_path.display(),
                ),
            NixpkgsProblem::InvalidPackageName { relative_package_dir, package_name } =>
                write!(
                    f,
                    "{}: Invalid package directory name \"{package_name}\", must be ASCII characters consisting of a-z, A-Z, 0-9, \"-\" or \"_\".",
                    relative_package_dir.display(),
                ),
            NixpkgsProblem::IncorrectShard { relative_package_dir, correct_relative_package_dir } =>
                write!(
                    f,
                    "{}: Incorrect directory location, should be {} instead.",
                    relative_package_dir.display(),
                    correct_relative_package_dir.display(),
                ),
            NixpkgsProblem::PackageNixNonExistent { relative_package_dir } =>
                write!(
                    f,
                    "{}: Missing required \"{PACKAGE_NIX_FILENAME}\" file.",
                    relative_package_dir.display(),
                ),
            NixpkgsProblem::PackageNixDir { relative_package_dir } =>
                write!(
                    f,
                    "{}: \"{PACKAGE_NIX_FILENAME}\" must be a file.",
                    relative_package_dir.display(),
                ),
            NixpkgsProblem::UndefinedAttr { relative_package_file, package_name } =>
                write!(
                    f,
                    "pkgs.{package_name}: This attribute is not defined but it should be defined automatically as {}",
                    relative_package_file.display()
                ),
            NixpkgsProblem::WrongCallPackage { relative_package_file, package_name } =>
                write!(
                    f,
                    "pkgs.{package_name}: This attribute is manually defined (most likely in pkgs/top-level/all-packages.nix), which is only allowed if the definition is of the form `pkgs.callPackage {} {{ ... }}` with a non-empty second argument.",
                    relative_package_file.display()
                ),
            NixpkgsProblem::WrongCallPackagePath { package_name, file, line, column, actual_path, expected_path } =>
                writedoc! {
                    f,
                    "
                    - Because {} exists, the attribute `pkgs.{package_name}` must be defined like

                        {package_name} = callPackage {} {{ /* ... */ }};

                      This is however not the case: The first `callPackage` argument is the wrong path.
                      It is defined in {}:{}:{} as

                        {package_name} = callPackage {} {{ /* ... */ }};",
                    structure::relative_dir_for_package(package_name).display(),
                    create_path_expr(file, expected_path),
                    file.display(), line, column,
                    create_path_expr(file, actual_path),
                },
            NixpkgsProblem::NonSyntacticCallPackage { package_name, file, line, column, definition } => {
                // This is needed such multi-line definitions don't look odd
                // A bit round-about, but it works and we might not need anything more complicated
                let definition_indented =
                    // The entire code should be indented 4 spaces
                    textwrap::indent(
                        // But first we want to strip the code's natural indentation
                        &textwrap::dedent(
                            // The definition _doesn't_ include the leading spaces, but we can
                            // recover those from the column
                            &format!("{}{definition}",
                            " ".repeat(column - 1)),
                        ),
                        "    ",
                    );
                writedoc!(
                    f,
                    "
                    - Because {} exists, the attribute `pkgs.{package_name}` must be defined like

                        {package_name} = callPackage {} {{ /* ... */ }};

                      This is however not the case.
                      It is defined in {}:{} as

                    {}",
                    structure::relative_dir_for_package(package_name).display(),
                    structure::relative_file_for_package(package_name).display(),
                    file.display(),
                    line,
                    definition_indented,
                )
            }
            NixpkgsProblem::NonDerivation { relative_package_file, package_name } =>
                write!(
                    f,
                    "pkgs.{package_name}: This attribute defined by {} is not a derivation",
                    relative_package_file.display()
                ),
            NixpkgsProblem::OutsideSymlink { relative_package_dir, subpath } =>
                write!(
                    f,
                    "{}: Path {} is a symlink pointing to a path outside the directory of that package.",
                    relative_package_dir.display(),
                    subpath.display(),
                ),
            NixpkgsProblem::UnresolvableSymlink { relative_package_dir, subpath, io_error } =>
                write!(
                    f,
                    "{}: Path {} is a symlink which cannot be resolved: {io_error}.",
                    relative_package_dir.display(),
                    subpath.display(),
                ),
            NixpkgsProblem::PathInterpolation { relative_package_dir, subpath, line, text } =>
                write!(
                    f,
                    "{}: File {} at line {line} contains the path expression \"{}\", which is not yet supported and may point outside the directory of that package.",
                    relative_package_dir.display(),
                    subpath.display(),
                    text
                ),
            NixpkgsProblem::SearchPath { relative_package_dir, subpath, line, text } =>
                write!(
                    f,
                    "{}: File {} at line {line} contains the nix search path expression \"{}\" which may point outside the directory of that package.",
                    relative_package_dir.display(),
                    subpath.display(),
                    text
                ),
            NixpkgsProblem::OutsidePathReference { relative_package_dir, subpath, line, text } =>
                write!(
                    f,
                    "{}: File {} at line {line} contains the path expression \"{}\" which may point outside the directory of that package.",
                    relative_package_dir.display(),
                    subpath.display(),
                    text,
                ),
            NixpkgsProblem::UnresolvablePathReference { relative_package_dir, subpath, line, text, io_error } =>
                write!(
                    f,
                    "{}: File {} at line {line} contains the path expression \"{}\" which cannot be resolved: {io_error}.",
                    relative_package_dir.display(),
                    subpath.display(),
                    text,
                ),
            NixpkgsProblem::MovedOutOfByName { package_name, call_package_path, empty_arg } => {
                let call_package_arg =
                    if let Some(path) = &call_package_path {
                        format!("./{}", path.display())
                    } else {
                        "...".into()
                    };
                if *empty_arg {
                    write!(
                        f,
                        "pkgs.{package_name}: This top-level package was previously defined in {}, but is now manually defined as `callPackage {call_package_arg} {{ }}` (e.g. in `pkgs/top-level/all-packages.nix`). Please move the package back and remove the manual `callPackage`.",
                        structure::relative_file_for_package(package_name).display(),
                        )
                } else {
                    // This can happen if users mistakenly assume that for custom arguments,
                    // pkgs/by-name can't be used.
                    write!(
                        f,
                        "pkgs.{package_name}: This top-level package was previously defined in {}, but is now manually defined as `callPackage {call_package_arg} {{ ... }}` (e.g. in `pkgs/top-level/all-packages.nix`). While the manual `callPackage` is still needed, it's not necessary to move the package files.",
                        structure::relative_file_for_package(package_name).display(),
                        )
                }
            },
            NixpkgsProblem::NewPackageNotUsingByName { package_name, call_package_path, empty_arg } => {
                let call_package_arg =
                    if let Some(path) = &call_package_path {
                        format!("./{}", path.display())
                    } else {
                        "...".into()
                    };
                let extra =
                    if *empty_arg {
                        "Since the second `callPackage` argument is `{ }`, no manual `callPackage` (e.g. in `pkgs/top-level/all-packages.nix`) is needed anymore."
                    } else {
                        "Since the second `callPackage` argument is not `{ }`, the manual `callPackage` (e.g. in `pkgs/top-level/all-packages.nix`) is still needed."
                    };
                write!(
                    f,
                    "pkgs.{package_name}: This is a new top-level package of the form `callPackage {call_package_arg} {{ }}`. Please define it in {} instead. See `pkgs/by-name/README.md` for more details. {extra}",
                    structure::relative_file_for_package(package_name).display(),
                )
            },
            NixpkgsProblem::InternalCallPackageUsed { attr_name } =>
                write!(
                    f,
                    "pkgs.{attr_name}: This attribute is defined using `_internalCallByNamePackageFile`, which is an internal function not intended for manual use.",
                ),
            NixpkgsProblem::CannotDetermineAttributeLocation { attr_name } =>
                write!(
                    f,
                    "pkgs.{attr_name}: Cannot determine the location of this attribute using `builtins.unsafeGetAttrPos`.",
                ),
       }
    }
}

/// Creates a Nix path expression that when put into Nix file `from_file`, would point to the `to_file`.
fn create_path_expr(from_file: impl AsRef<Path>, to_file: impl AsRef<Path>) -> String {
    // FIXME: Clean these unwrap's up
    // But these should never trigger because we only call this function with relative paths, and only
    // with `from_file` being an actual file (so there's always a parent)
    let from = RelativePath::from_path(&from_file)
        .unwrap()
        .parent()
        .unwrap();
    let to = RelativePath::from_path(&to_file).unwrap();
    let rel = from.relative(to);
    format!("./{rel}")
}
