use openai_responses_api::{
    Auth, Client, Config, OrganizationId, ProjectId, ResponsesError, Secret, Url,
};

#[test]
fn client_builds_for_both_auth_kinds() -> Result<(), ResponsesError> {
    let mut config = Config::new(Auth::Bearer(Secret("token".to_owned())))?;
    config.base_url = Url::parse("http://127.0.0.1:8000/v1/")?;
    config.organization = Some(OrganizationId("org-1".to_owned()));
    config.project = Some(ProjectId("proj-1".to_owned()));
    Client::new(config)?;
    Client::new(Config::new(Auth::ApiKey(Secret("azure-key".to_owned())))?)?;
    Ok(())
}

#[test]
fn bad_header_value_is_an_error() -> Result<(), ResponsesError> {
    let built = Client::new(Config::new(Auth::Bearer(Secret("bad\ntoken".to_owned())))?);
    assert!(matches!(built, Err(ResponsesError::Header(..))));
    let mut config = Config::new(Auth::Bearer(Secret("ok".to_owned())))?;
    config.project = Some(ProjectId("bad\nproject".to_owned()));
    assert!(matches!(
        Client::new(config),
        Err(ResponsesError::Header(..))
    ));
    Ok(())
}

#[test]
fn base_url_without_path_room_is_an_error() -> Result<(), ResponsesError> {
    let mut config = Config::new(Auth::Bearer(Secret("k".to_owned())))?;
    config.base_url = Url::parse("data:text/plain,x")?;
    assert!(matches!(
        Client::new(config),
        Err(ResponsesError::CannotBeABase(..))
    ));
    Ok(())
}
