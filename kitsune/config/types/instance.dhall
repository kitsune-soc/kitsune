let FederationFilter = ./federation_filter.dhall

in  { name : Text
    , description : Text
    , character_limit : Natural
    , registrations_open : Bool
    , federation_filter : FederationFilter
    }
