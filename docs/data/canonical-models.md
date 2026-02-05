# Canonical models

## Intent

Define the core business entities used for data unification.

## Entity overview

```mermaid
erDiagram
  CUSTOMER ||--o{ TRANSACTION : places
  CUSTOMER ||--o{ ASSET : owns
  PRODUCT ||--o{ TRANSACTION : includes

  CUSTOMER {
    string id
    string name
    string email
    string segment
  }

  PRODUCT {
    string id
    string name
    string category
  }

  TRANSACTION {
    string id
    string status
    float amount
    string currency
    date occurred_at
  }

  ASSET {
    string id
    string type
    string status
  }
```

## Notes

- This is a minimum viable model for V1 and should be expanded per domain.
- Attribute definitions and constraints will live in the schema repo.

## Open questions

- Which system becomes source of truth per entity?
- How do we version canonical model changes?
