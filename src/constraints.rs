use crate::{array, exact, number, text, Error, Result, MULTI, SINGLE};
use serde_json::Value;
use std::collections::{HashSet, VecDeque};
const AMOUNT: u64 = 9_007_199_254_740_991;
fn err() -> Error {
    Error::reject("invalid_scoped_schema")
}
fn amount(v: &Value) -> Result<u64> {
    number(v, 0, AMOUNT)
}
fn quantity(v: &Value) -> Result<u64> {
    number(v, 1, 1_000_000)
}
fn totals(v: &Value, n: u64) -> Result<()> {
    let a = array(v, 2, 2)?;
    let mut types = HashSet::new();
    for e in a {
        exact(e, &["amount", "type"])?;
        let t = text(&e["type"], 256)?;
        if !["subtotal", "total"].contains(&t) || !types.insert(t) {
            return Err(err());
        }
        if amount(&e["amount"])? != n {
            return Err(Error::reject("amount_mismatch"));
        }
    }
    Ok(())
}
fn merchant(v: &Value) -> Result<()> {
    exact(v, &["id", "name"])?;
    text(&v["id"], 256)?;
    text(&v["name"], 512)?;
    Ok(())
}
fn unique(v: &[Value]) -> Result<()> {
    let mut ids = HashSet::new();
    for e in v {
        if !ids.insert(text(&e["id"], 256)?) {
            return Err(err());
        }
    }
    Ok(())
}
pub fn checkout(c: &Value, profile: &str) -> Result<u64> {
    let max = match profile {
        SINGLE => 1,
        MULTI => 8,
        _ => return Err(Error::reject("unsupported_scoped_profile")),
    };
    exact(
        c,
        &[
            "currency",
            "id",
            "line_items",
            "links",
            "merchant",
            "status",
            "totals",
        ],
    )?;
    text(&c["id"], 256)?;
    merchant(&c["merchant"])?;
    if !["USD", "EUR", "JPY"].contains(&text(&c["currency"], 3)?)
        || c["status"] != "ready_for_complete"
        || c["links"] != serde_json::json!([])
    {
        return Err(Error::reject("unsupported_scoped_profile"));
    }
    let lines = array(&c["line_items"], 1, max)?;
    unique(lines)?;
    let mut sum = 0u64;
    for l in lines {
        exact(l, &["id", "item", "quantity", "totals"])?;
        text(&l["id"], 256)?;
        let q = quantity(&l["quantity"])?;
        let i = &l["item"];
        exact(i, &["id", "price", "title"])?;
        text(&i["id"], 256)?;
        text(&i["title"], 512)?;
        let a = amount(&i["price"])?
            .checked_mul(q)
            .filter(|v| *v <= AMOUNT)
            .ok_or_else(err)?;
        totals(&l["totals"], a)?;
        sum = sum
            .checked_add(a)
            .filter(|v| *v <= AMOUNT)
            .ok_or_else(err)?;
    }
    totals(&c["totals"], sum)?;
    Ok(sum)
}
pub fn check(open: &Value, c: &Value, profile: &str, currency: &str, total: u64) -> Result<()> {
    if checkout(c, profile)? != total {
        return Err(Error::reject("amount_mismatch"));
    }
    if c["currency"] != currency {
        return Err(Error::reject("currency_mismatch"));
    }
    let a = array(&open["constraints"], 2, 2)?;
    exact(&a[0], &["items", "type"])?;
    exact(&a[1], &["allowed", "type"])?;
    if a[0]["type"] != "checkout.line_items" || a[1]["type"] != "checkout.allowed_merchants" {
        return Err(Error::reject("unsupported_constraint"));
    }
    let m = &array(&a[1]["allowed"], 1, 1)?[0];
    merchant(m)?;
    if c["merchant"] != *m {
        return Err(Error::reject("constraint_mismatch"));
    }
    let req = array(&a[0]["items"], 1, if profile == SINGLE { 1 } else { 8 })?;
    unique(req)?;
    for r in req {
        exact(r, &["acceptable_items", "id", "quantity"])?;
        quantity(&r["quantity"])?;
        let candidates = array(
            &r["acceptable_items"],
            1,
            if profile == SINGLE { 1 } else { 8 },
        )?;
        unique(candidates)?;
        for i in candidates {
            exact(i, &["id", "title"])?;
            text(&i["title"], 512)?;
        }
    }
    let lines = array(&c["line_items"], 1, 8)?;
    if profile == SINGLE && req[0]["acceptable_items"][0]["title"] != lines[0]["item"]["title"] {
        return Err(Error::reject("constraint_mismatch"));
    }
    let demand = req
        .iter()
        .map(|r| r["quantity"].as_u64().unwrap_or(0))
        .sum::<u64>();
    if lines
        .iter()
        .map(|l| l["quantity"].as_u64().unwrap_or(0))
        .sum::<u64>()
        != demand
    {
        return Err(Error::reject("constraint_mismatch"));
    }
    let sink = 1 + req.len() + lines.len();
    let mut residual = vec![vec![0u64; sink + 1]; sink + 1];
    for (i, r) in req.iter().enumerate() {
        residual[0][i + 1] = quantity(&r["quantity"])?;
        for (j, l) in lines.iter().enumerate() {
            if array(&r["acceptable_items"], 1, 8)?
                .iter()
                .any(|v| v["id"] == l["item"]["id"])
            {
                residual[i + 1][1 + req.len() + j] = demand
            }
        }
    }
    for (j, l) in lines.iter().enumerate() {
        residual[1 + req.len() + j][sink] = quantity(&l["quantity"])?;
    }
    let mut flow = 0;
    loop {
        let mut parent = vec![None; sink + 1];
        parent[0] = Some(0);
        let mut queue = VecDeque::from([0]);
        while let Some(u) = queue.pop_front() {
            for (v, predecessor) in parent.iter_mut().enumerate() {
                if residual[u][v] > 0 && predecessor.is_none() {
                    *predecessor = Some(u);
                    queue.push_back(v);
                }
            }
        }
        if parent[sink].is_none() {
            break;
        }
        let mut n = demand;
        let mut v = sink;
        while v != 0 {
            let u = parent[v].ok_or_else(err)?;
            n = n.min(residual[u][v]);
            v = u;
        }
        v = sink;
        while v != 0 {
            let u = parent[v].ok_or_else(err)?;
            residual[u][v] -= n;
            residual[v][u] += n;
            v = u;
        }
        flow += n;
    }
    if flow != demand {
        return Err(Error::reject("constraint_mismatch"));
    }
    Ok(())
}
