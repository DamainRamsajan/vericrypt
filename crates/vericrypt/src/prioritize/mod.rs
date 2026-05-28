use crate::errors::VeriCryptError; use crate::graph::CryptoGraph;
use crate::types::{ExposureResult, MigrationPhase};
pub fn generate_roadmap(er: &ExposureResult, _g: &CryptoGraph) -> Result<Vec<MigrationPhase>, VeriCryptError> {
    let mut e: Vec<_> = er.shapley_values.iter().map(|(k,v)|(*k,*v)).collect();
    e.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let t = e.len(); let p1 = if t>0{t/3}else{0}; let p2 = if t>0{2*t/3}else{0};
    Ok(e.iter().enumerate().map(|(i,(id,_))| {
        let ph = if i<p1{1}else if i<p2{2}else{3};
        MigrationPhase{phase:ph,asset_id:*id,current_algorithm:"Classified".into(),recommended_replacement:"ML-DSA/SLH-DSA".into(),regulatory_reference:format!("DORA Art.12.3 Phase {}",ph),estimated_complexity:match ph{1=>"High".into(),2=>"Medium".into(),_=>"Standard".into()}}
    }).collect())
}
