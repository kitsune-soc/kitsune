let Kitsune = ./search/kitsune.dhall

let Meilisearch = ./search/meilisearch.dhall

in  < Kitsune : Kitsune | Meilisearch : Meilisearch | Sql | None >
