use crate::parser;

#[derive(Debug)]
pub enum IPFSError {
    GenericError
}

pub fn query(_package: &str) -> Result<parser::PkgInfo, IPFSError> {
    return Ok(parser::PkgInfo {
        name:      "zzz".to_string(),
        version:   "zzz".to_string(),
        ipfs_hash: "zzz".to_string()
    });
}

pub fn update(_config: &Vec<parser::PkgInfo>) -> Result<(), IPFSError> {
    Ok(())
}

pub fn download(_name: &str) -> Result<(), IPFSError> {
    Ok(())
}

fn add_pkg(_package: &parser::PkgInfo) -> Result<(), IPFSError> {
    Ok(())
}

pub fn add(pkgs: &Vec<parser::PkgInfo>) -> Result<(), IPFSError> {

    for pkg in pkgs {
        let res = add_pkg(pkg);

        if res.is_err() {
            return Err(res.err().unwrap());
        }
    }

    Ok(())
}
