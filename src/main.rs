#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate cf_dist_utils;
extern crate cf_functions;
extern crate fang_oost;
extern crate lambda_http;
extern crate lambda_runtime as runtime;
extern crate num_complex;
extern crate rayon;
use self::rayon::prelude::*;
use lambda_http::{lambda, Body, IntoResponse, Request, Response};
use runtime::{error::HandlerError, Context};
use std::error::Error;

fn build_response(code: u16, body: &str) -> impl IntoResponse {
    Response::builder()
        .status(code)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Credentials", "true")
        .body::<Body>(body.into())
        .unwrap()
}
fn construct_error(e_message: &str) -> String {
    json!({ "err": e_message }).to_string()
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    t: f64,
    a: f64,
    sigma: f64,
    lambda: f64,
    correlation: f64,
    alpha: f64,
    mu: f64,
    c: f64,
    num_u: usize,
    num_ode: usize,
}

#[derive(Debug, Serialize)]
struct Element {
    density: f64,
    at_point: f64,
}

const NUM_X: usize = 512;
const X_MIN: f64 = 0.0;

fn main() -> Result<(), Box<dyn Error>> {
    lambda!(ops_faas_wrapper);
    Ok(())
}
fn ops_faas_wrapper(event: Request, _ctx: Context) -> Result<impl IntoResponse, HandlerError> {
    match ops_faas(event) {
        Ok(res) => Ok(build_response(200, &json!(res).to_string())),
        Err(e) => Ok(build_response(400, &construct_error(&e.to_string()))),
    }
}

fn ops_faas(event: Request) -> Result<Vec<Element>, Box<dyn Error>> {
    let parameters: Parameters = serde_json::from_reader(event.body().as_ref())?;
    Ok(get_density(parameters))
}
fn compute_x_max(lambda: f64, mu: f64, c: f64, t: f64) -> f64 {
    lambda * (mu + 35.0 * c) * t
}
fn get_density(parameters: Parameters) -> Vec<Element> {
    let Parameters {
        t,
        a,
        sigma,
        lambda,
        correlation,
        alpha,
        mu,
        c,
        num_u,
        num_ode,
    } = parameters;
    let v0 = 1.0; //made for ease
    let x_max = compute_x_max(lambda, mu, c, t);
    let cf = cf_functions::alpha_stable_leverage(
        t,
        v0,
        a,
        sigma,
        lambda,
        correlation,
        alpha,
        mu,
        c,
        num_ode,
    );
    fang_oost::get_density(
        X_MIN,
        x_max,
        fang_oost::get_x_domain(NUM_X, X_MIN, x_max),
        &fang_oost::get_discrete_cf(num_u, X_MIN, x_max, &cf),
    )
    .zip(fang_oost::get_x_domain(NUM_X, X_MIN, x_max))
    .map(|(density, at_point)| Element { density, at_point })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize() {
        let json_str="{\"t\":1.0,\"a\":0.4,\"numU\":128,\"sigma\":0.3,\"correlation\":0.9,\"alpha\":0.5, \"mu\":1300.0, \"c\":100.0, \"numOde\":128, \"lambda\":100.0}";
        let parameters: Parameters = serde_json::from_str(json_str).unwrap();
        assert_eq!(parameters.t, 1.0);
        assert_eq!(parameters.num_u, 128);
    }
    #[test]
    fn test_get_density() {
        let parameters = Parameters {
            t: 1.0,
            a: 0.4,
            num_u: 128,
            sigma: 0.4,
            alpha: 1.1,
            lambda: 100.0,
            c: 100.0,
            mu: 1300.0,
            correlation: 0.9,
            num_ode: 128,
        };
        let result = get_density(parameters);
        assert_eq!(result.len(), 512);
    }
}
