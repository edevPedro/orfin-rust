use sqlx::PgPool;

use crate::models::Category;

const HEURISTICS: &[(&str, &[&str])] = &[
    (
        "alimentacao",
        &[
            "ifood", "rappi", "uber eats", "mcdonald", "burger", "padaria", "mercado",
            "carrefour", "extra", "pao de acucar", "assai", "atacadao", "restaurante",
            "lanchonete", "starbucks", "cafe", "pizza", "churras",
        ],
    ),
    (
        "transporte",
        &[
            "uber", "99app", "99 ", "cabify", "posto", "shell", "ipiranga", "petrobras",
            "combustivel", "gasolina", "estacionamento", "pedagio", "metro", "onibus",
        ],
    ),
    (
        "moradia",
        &[
            "aluguel", "condominio", "energia", "enel", "cemig", "copel", "agua", "sabesp",
            "vivo fibra", "claro residencial", "tim live",
        ],
    ),
    (
        "assinaturas",
        &[
            "netflix", "spotify", "amazon prime", "disney", "hbo", "youtube premium",
            "apple.com/bill", "google one", "icloud", "microsoft", "adobe", "chatgpt", "openai",
        ],
    ),
    (
        "saude",
        &[
            "farmacia", "drogaria", "raia", "pacheco", "hospital", "clinica", "laboratorio",
            "unimed", "amil", "dentista",
        ],
    ),
    (
        "educacao",
        &[
            "escola", "faculdade", "universidade", "curso", "udemy", "alura", "coursera",
            "livraria", "papelaria",
        ],
    ),
    (
        "lazer",
        &[
            "cinema", "ingresso", "steam", "playstation", "xbox", "bar ", "show",
            "ingresso.com", "sympla", "hotel", "booking", "airbnb",
        ],
    ),
];

#[derive(Debug, sqlx::FromRow)]
struct UserRule {
    match_type: String,
    match_value: String,
    category_id: String,
}

pub async fn list_categories(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>("SELECT id, label FROM categories ORDER BY label")
        .fetch_all(pool)
        .await
}

pub async fn suggest_category(
    pool: &PgPool,
    user_id: &str,
    merchant: Option<&str>,
    description: Option<&str>,
    existing: Option<&str>,
) -> Result<String, sqlx::Error> {
    if let Some(category) = existing.filter(|value| !value.is_empty()) {
        return Ok(normalize_slug(category));
    }

    let haystack = format!(
        "{} {}",
        merchant.unwrap_or_default(),
        description.unwrap_or_default()
    )
    .to_lowercase();

    if let Some(category_id) = match_user_rule(pool, user_id, &haystack).await? {
        return Ok(category_id);
    }

    for (category, keywords) in HEURISTICS {
        if keywords.iter().any(|keyword| haystack.contains(keyword)) {
            return Ok((*category).to_string());
        }
    }

    Ok("outros".to_string())
}

async fn match_user_rule(
    pool: &PgPool,
    user_id: &str,
    haystack: &str,
) -> Result<Option<String>, sqlx::Error> {
    let rules = sqlx::query_as::<_, UserRule>(
        "SELECT match_type, match_value, category_id FROM user_category_rules WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rules.into_iter().find_map(|rule| {
        let needle = rule.match_value.to_lowercase();
        let matched = matches!(
            rule.match_type.as_str(),
            "merchant_contains" | "description_contains"
        ) && haystack.contains(&needle);
        matched.then_some(rule.category_id)
    }))
}

fn normalize_slug(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_whitespace() || ch == '-' {
                '_'
            } else {
                ch
            }
        })
        .collect()
}
