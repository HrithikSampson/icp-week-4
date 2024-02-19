use candid::{CandidType, Deserialize};
use std::cell::RefCell;
use serde_json::Value;
use ic_cdk::api::{call, management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
}};
use ic_cdk::query;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
 
}

#[derive(CandidType, Deserialize, Default, Debug, Clone)]
struct TvlData {
    tvl_json: String,
}
thread_local! {
    static TVL_DATA: RefCell<TvlData> = RefCell::new(TvlData::default());
}
#[ic_cdk_macros::update]
async fn fetch_defillama_tvl() -> Result<(), String> {
    let url = "https://api.llama.fi/v2/historicalChainTvl".to_string();
    let headers = vec![HttpHeader { name: "Accept".to_string(), value: "application/json".to_string() }];

    match http_request(CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        headers,
        max_response_bytes: None,
        body: None,
        transform: None
    }, 1_603_085_600).await {
        Ok((status_code,)) => {
            if status_code.status == 200 as usize {
                let body_str = String::from_utf8(status_code.body).map_err(|e| e.to_string())?;

                TVL_DATA.with(|tvl_data| {
                    *tvl_data.borrow_mut() = TvlData { tvl_json: body_str };
                });

                Ok(())
            } else {
                Err(format!("HTTP request failed with status code: {}", status_code.status))
            }
        },
        Err(e) => Err(format!("HTTP request failed: {:?}", e)),
    }
}

#[ic_cdk_macros::query]
fn get_stored_tvl_data() -> Result<String, String> {
    TVL_DATA.with(|tvl_data| {
        Ok(tvl_data.borrow().tvl_json.clone())
    })
}


#[ic_cdk_macros::init]
fn init() {
    ic_cdk::setup();
}


// This is required to generate the Candid interface automatically
ic_cdk_macros::export_candid!();
