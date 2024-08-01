#![allow(dead_code)]

use reading_liner::location::line_column;
use std::{collections::HashMap, hash::Hash, io, marker::PhantomData, mem, path, rc::Rc};

use crate::summon::Summoner;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileLocationId<P> {
    file_path: P,
    location: line_column::ZeroBased,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GoSymbol(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GoSymbolId<P> {
    path: P,
    symbol: GoSymbol,
}

fn summon_raw_by_file_path<P: AsRef<path::Path>>(id: P) -> io::Result<gosyn::ast::File> {
    let file =
        gosyn::parse_file(id.as_ref()).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    Ok(file)
}

fn summon_by_file_path(id: impl AsRef<path::Path>) -> io::Result<GoFile> {
    let gosyn_file = summon_raw_by_file_path(id)?;
    Ok(GoFile::from(gosyn_file))
}

// #[derive(Default, Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub struct File {
//     pub path: PathBuf,
//     pub line_info: Vec<usize>,
//     pub docs: Vec<Rc<Comment>>,
//     pub pkg_name: Ident,
//     pub imports: Vec<Import>,
//     pub decl: Vec<Declaration>,
//     pub comments: Vec<Rc<Comment>>,
// }
#[derive(Debug)]
pub struct GoFile {
    pub pkg_name: String,
    pub line_info: Vec<usize>,
    pub docs: Vec<Rc<gosyn::ast::Comment>>,
    pub imports: HashMap<String, String>,
    pub decls: HashMap<String, gosyn::ast::Declaration>,
    pub comments: Vec<Rc<gosyn::ast::Comment>>,
}

impl GoFile {
    pub fn from_gosyn_file(gosyn_file: gosyn::ast::File) -> Self {
        let imports = Self::extract_imports(gosyn_file.imports);
        let decls = Self::extract_decls(gosyn_file.decl);

        Self {
            pkg_name: gosyn_file.pkg_name.name,
            line_info: gosyn_file.line_info,
            docs: gosyn_file.docs,
            imports,
            decls,
            comments: gosyn_file.comments,
        }
    }

    fn extract_decls(
        decls: Vec<gosyn::ast::Declaration>,
    ) -> HashMap<String, gosyn::ast::Declaration> {
        let mut ans = HashMap::new();
        for decl in decls {
            match decl {
                gosyn::ast::Declaration::Function(ref func_decl) => {
                    let name = func_decl.name.name.clone();
                    ans.insert(name, decl);
                }
                gosyn::ast::Declaration::Type(mut decl) => {
                    let ds = mem::take(&mut decl.specs);
                    for d in ds {
                        let mut header = decl.clone();
                        let name = d.name.name.clone();
                        header.specs = vec![d];
                        ans.insert(name, gosyn::ast::Declaration::Type(header));
                    }
                }
                gosyn::ast::Declaration::Const(mut decl) => {
                    let specs = mem::take(&mut decl.specs);
                    for spec in specs {
                        let mut header = decl.clone();
                        let names = spec.name.clone();
                        header.specs = vec![spec];
                        for name in names {
                            ans.insert(name.name, gosyn::ast::Declaration::Const(header.clone()));
                        }
                    }
                }
                gosyn::ast::Declaration::Variable(mut decl) => {
                    let specs = mem::take(&mut decl.specs);
                    for spec in specs {
                        let mut header = decl.clone();
                        let names = spec.name.clone();
                        header.specs = vec![spec];
                        for name in names {
                            ans.insert(
                                name.name,
                                gosyn::ast::Declaration::Variable(header.clone()),
                            );
                        }
                    }
                }
            };
        }
        ans
    }

    fn extract_imports(imports: Vec<gosyn::ast::Import>) -> HashMap<String, String> {
        let mut ans = HashMap::new();
        for import in imports {
            let name = if let Some(name) = import.name {
                name.name
            } else {
                let cells = import.path.value.split("/");
                cells.last().expect("invalid import path").to_owned()
            };

            ans.insert(name, import.path.value);
        }
        ans
    }
}

impl From<gosyn::ast::File> for GoFile {
    fn from(value: gosyn::ast::File) -> Self {
        Self::from_gosyn_file(value)
    }
}

pub struct GoFileSummoner<P> {
    _data: PhantomData<P>,
}

impl<P> GoFileSummoner<P> {
    pub fn new() -> Self {
        Self { _data: PhantomData }
    }
}

impl<P> Default for GoFileSummoner<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: AsRef<path::Path>> Summoner<GoFile> for GoFileSummoner<P> {
    type Id = P;
    type Err = io::Error;

    fn summon(&self, id: Self::Id) -> Result<GoFile, Self::Err> {
        summon_by_file_path(id)
    }
}

pub struct GoDeclSummoner<P> {
    _data: PhantomData<P>,
}

impl<P> GoDeclSummoner<P> {
    pub fn new() -> Self {
        Self { _data: PhantomData }
    }
}

impl<P> Default for GoDeclSummoner<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: AsRef<path::Path>> Summoner<gosyn::ast::Declaration> for GoDeclSummoner<P> {
    type Id = GoSymbolId<P>;
    type Err = io::Error;

    fn summon(&self, id: Self::Id) -> Result<gosyn::ast::Declaration, Self::Err> {
        let file = summon_by_file_path(id.path)?;
        if let Some(decl) = file.decls.get(&id.symbol.0) {
            return Ok(decl.clone());
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("symbol {} not found", id.symbol.0),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::summon::CachedSummoner;

    #[test]
    fn test_summon_gofile() {
        let path = "/Users/comcx/Workspace/Repo/void/main.go";
        let summoner = GoFileSummoner::new();
        let summoner = CachedSummoner::new(summoner);
        let file = Summoner::<Rc<GoFile>>::summon(&summoner, path);
        dbg!(file.unwrap());

        let retrieve = summoner.summon(path);
        dbg!(retrieve.unwrap());
    }

    #[test]
    fn test_summon_gosymbol() {
        let path = "/Users/comcx/Workspace/Repo/void/main.go";
        let summoner = GoDeclSummoner::new();
        let summoner = CachedSummoner::new(summoner);

        let id = GoSymbolId {
            path,
            symbol: GoSymbol("main".to_owned()),
        };
        let file = summoner.summon(id.clone());
        dbg!(file.unwrap());

        let retrieve = summoner.summon(id);
        dbg!(retrieve.unwrap());
    }
}
