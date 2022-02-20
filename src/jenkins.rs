use crate::github::Artifact;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct SHA1(String);

#[derive(Deserialize, Serialize)]
struct Build {
    artifacts: Vec<BuildArtifact>,
}

#[derive(Deserialize, Serialize)]
pub struct BuildArtifact {
    #[serde(alias = "fileName")]
    file_name: String,
}

impl Artifact for BuildArtifact {
    fn name(&self) -> &str {
        &self.file_name
    }
}

pub fn get_latest_sha() -> Result<String> {
    let url = "https://ci.mcofficer.me/job/EndlessSky-continuous-bitar/lastSuccessfulBuild/api/xml?xpath=/*/*/lastBuiltRevision/SHA1";

    let res = ureq::get(url).call()?;
    let sha: SHA1 = serde_xml_rs::from_str(&res.into_string()?)?;
    info!("Got new version from Jenkins: {}", sha.0);
    Ok(sha.0)
}

pub fn get_latest_artifacts() -> Result<Vec<BuildArtifact>> {
    let url = "https://ci.mcofficer.me/job/EndlessSky-continuous-bitar/lastBuild/api/json?tree=artifacts[*]";

    let res = ureq::get(url).call()?;
    let build: Build = res.into_json()?;
    Ok(build.artifacts)
}
