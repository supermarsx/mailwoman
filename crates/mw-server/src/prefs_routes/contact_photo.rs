use super::*;
use base64::Engine as _;
use tokio::sync::Semaphore;

static DECODERS: Semaphore = Semaphore::const_new(2);

pub(super) fn router() -> Router<AppState> {
    Router::new().route("/api/account/contact-photo", axum::routing::post(photo))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request { address: String }

fn embedded(line: &str) -> Option<Vec<u8>> {
    if line.len()>262144 { return None; }
    let (head,value)=line.trim_end_matches(['\r','\n']).split_once(':')?;
    if !head.split(';').next()?.rsplit('.').next()?.eq_ignore_ascii_case("PHOTO") { return None; }
    let encoded=if let Some(data)=value.strip_prefix("data:") {
        let (metadata,encoded)=data.split_once(',')?;
        if !["image/png;base64","image/jpeg;base64"].iter().any(|allowed|metadata.eq_ignore_ascii_case(allowed)) { return None; }
        encoded
    } else {
        if !head.split(';').skip(1).any(|parameter|parameter.split_once('=').is_some_and(|(key,value)|key.eq_ignore_ascii_case("ENCODING")&&matches!(value.trim_matches('"').to_ascii_lowercase().as_str(),"b"|"base64"))) {return None;}
        value
    };
    let bytes=base64::engine::general_purpose::STANDARD.decode(encoded).ok()?;
    if bytes.len()>128*1024 { return None; }
    let (w,h)=crate::branding_routes::dimensions(&bytes)?;
    (w>0&&h>0&&w<=512&&h<=512).then_some(bytes)
}

async fn photo(State(state):State<AppState>,headers:HeaderMap,Json(body):Json<Request>)->Response {
    let session=match session_of(&state,&headers).await {Ok(value)=>value,Err(response)=>return response};
    let address=match mw_store::receipts::normalize_sender(&body.address) {Ok(value)=>value,Err(_)=>return bad_request("one mailbox address is required")};
    let permitted=match crate::settings_routes::effective_for_session(&state,&session,"contacts.avatarMode").await {
        Ok(Some(value))=>value.value.as_str()==Some("contacts"),
        Ok(None)=>false,
        Err(_)=>return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if !permitted {return StatusCode::NO_CONTENT.into_response();}
    let Some(engine)=&state.engine else {return StatusCode::NO_CONTENT.into_response();};
    let line=match engine.contact_photo(&session.account_id,&address).await {
        Ok(Some(line))=>line,
        Ok(None)=>return StatusCode::NO_CONTENT.into_response(),
        Err(_)=>return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let Some(bytes)=embedded(&line) else {return StatusCode::NO_CONTENT.into_response();};
    let Ok(permit)=DECODERS.try_acquire() else {return StatusCode::TOO_MANY_REQUESTS.into_response();};
    let result=tokio::task::spawn_blocking(move||{let _permit=permit;mw_render::media_jail::reencode_image(&bytes)}).await;
    match result {
        Ok(Ok(png)) if png.len()<=1024*1024=>([("content-type","image/png"),("cache-control","private, no-store"),("x-content-type-options","nosniff"),("content-security-policy","default-src 'none'; sandbox")],png).into_response(),
        _=>StatusCode::NO_CONTENT.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_bounded_embedded_raster_photos_are_accepted() {
        let mut png=b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();png.extend_from_slice(&1_u32.to_be_bytes());png.extend_from_slice(&1_u32.to_be_bytes());
        let encoded=base64::engine::general_purpose::STANDARD.encode(&png);
        assert_eq!(embedded(&format!("PHOTO;ENCODING=b;TYPE=PNG:{encoded}\r\n")),Some(png.clone()));
        assert_eq!(embedded(&format!("item1.PHOTO:data:image/png;base64,{encoded}")),Some(png));
        for value in ["PHOTO:https://example.test/private.png","PHOTO:file:///etc/passwd","PHOTO:data:image/svg+xml;base64,PHN2Zy8+","PHOTO;ENCODING=b:!!!","NOTE;ENCODING=b:AAAA"] {assert!(embedded(value).is_none());}
        assert!(embedded(&format!("PHOTO;ENCODING=b:{}","A".repeat(262144))).is_none());
    }
}
