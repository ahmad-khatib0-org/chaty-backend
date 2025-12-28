use chaty_proto::envoy_service::auth::v3::{
  authorization_server::Authorization, CheckRequest, CheckResponse,
};
use chaty_utils::time::time_get_seconds;
use tonic::{Code, Request, Response, Status};
use tracing::{info, instrument, Span};

use crate::utils::network::extract_jwt_claims_and_token;

use super::{
  hydra::{HydraClient, HydraValidation},
  redis::{RedisCheck, RedisClient},
  response::CheckResponseExt,
  routes::ROUTES,
  Controller,
};

#[tonic::async_trait]
impl Authorization for Controller {
  #[doc = " Performs authorization check based on the attributes associated with the"]
  #[doc = " incoming request, and returns status `OK` or not `OK`."]
  #[instrument(skip(self, request), fields(path = "", token_present = false))]
  async fn check(&self, request: Request<CheckRequest>) -> Result<Response<CheckResponse>, Status> {
    let start = std::time::Instant::now();
    let ctx = self.get_context(request.get_ref()).await;
    let req = request.get_ref();
    let lang = ctx.accept_language();

    let current_span = Span::current();
    info!("Authorization check started");

    let path = req
      .attributes
      .as_ref()
      .and_then(|a| a.request.as_ref())
      .and_then(|r| r.http.as_ref())
      .map(|h| h.path.clone())
      .ok_or_else(|| Status::new(Code::NotFound, Self::not_found_msg(lang)))?;

    current_span.record("path", &path);
    info!("Checking path: {}", path);

    let protected = match ROUTES.get(&path) {
      Some(res) => *res,
      None => {
        self.metrics.record_auth_denied("route_not_found");
        return Err(Status::new(Code::NotFound, Self::not_found_msg(lang)));
      }
    };

    if !protected {
      self.metrics.record_auth_allowed();
      let duration = start.elapsed().as_secs_f64();
      self.metrics.observe_request_duration(duration);
      info!("Authorization allowed for unprotected route - duration: {:.2}ms", duration * 1000.0);
      return Ok(self.response_ok(&ctx, &request, None).await);
    }

    let (claims, token) = extract_jwt_claims_and_token(&request);
    current_span.record("token_present", !token.is_empty());
    info!("Token present: {}", !token.is_empty());

    // the token id must be present, for a protected route
    if token.is_empty() {
      self.metrics.record_auth_denied("empty_token");
      let duration = start.elapsed().as_secs_f64();
      self.metrics.observe_request_duration(duration);
      info!("Authorization denied: empty token - duration: {:.2}ms", duration * 1000.0);
      return Ok(Response::new(CheckResponse::denied(&Self::invalid_token_msg(lang))));
    }

    match self.redis.check_token(&token).await {
      Ok(RedisCheck::Revoked(_reason)) => {
        self.metrics.record_auth_denied("token_revoked");
        self.metrics.record_token_revoked();
        let duration = start.elapsed().as_secs_f64();
        self.metrics.observe_request_duration(duration);
        info!("Authorization denied: token revoked - duration: {:.2}ms", duration * 1000.0);
        return Ok(Response::new(CheckResponse::denied(&Self::invalid_token_msg(lang))));
      }
      Ok(RedisCheck::Allowed { status }) => {
        let now = time_get_seconds();
        let needs_hydra = match status {
          Some(st) => now as i64 - st.last_checked > 300,
          None => true,
        };
        info!("Redis check passed - needs_hydra: {}", needs_hydra);

        if needs_hydra {
          info!("Validating token with Hydra");
          match self.hydra.validate_token(&token).await {
            Ok(HydraValidation::Valid { sub: _, exp: _ }) => {
              self.metrics.record_hydra_validation();
              self.metrics.record_token_check_success();
              self.redis.mark_checked_ok(&token).await.ok();
              self.metrics.record_auth_allowed();
              let duration = start.elapsed().as_secs_f64();
              self.metrics.observe_request_duration(duration);
              info!(
                "Authorization allowed (Hydra validated) - duration: {:.2}ms",
                duration * 1000.0
              );
              return Ok(self.response_ok(&ctx, &request, Some(claims)).await);
            }
            Ok(HydraValidation::Invalid(_reason)) => {
              self.metrics.record_hydra_validation_failure();
              self.redis.revoke_token(&token).await.ok();
              self.metrics.record_auth_denied("invalid_token");
              let duration = start.elapsed().as_secs_f64();
              self.metrics.observe_request_duration(duration);
              info!(
                "Authorization denied: Hydra validation failed - duration: {:.2}ms",
                duration * 1000.0
              );
              return Ok(Response::new(CheckResponse::denied(&Self::invalid_token_msg(lang))));
            }
            Err(_err) => {
              self.metrics.record_hydra_validation_failure();
              info!("Authorization error: Hydra validation error");
              return Err(Status::internal(Self::int_err_msg(lang)));
            }
          }
        } else {
          self.metrics.record_auth_allowed();
          let duration = start.elapsed().as_secs_f64();
          self.metrics.observe_request_duration(duration);
          info!("Authorization allowed (cached) - duration: {:.2}ms", duration * 1000.0);
          return Ok(self.response_ok(&ctx, &request, Some(claims)).await); // Cached as valid
        }
      }
      Err(_err) => {
        self.metrics.record_auth_denied("redis_check_failed");
        self.metrics.record_token_check_failure();
        info!("Authorization error: Redis check failed");
        return Err(Status::internal(Self::int_err_msg(lang)));
      }
    }
  }
}
