let Allow = ./federation_filter/allow.dhall

let Deny = ./federation_filter/deny.dhall

in  < Allow : Allow | Deny : Deny >
