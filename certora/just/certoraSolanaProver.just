# expects that the following variables are defined
# project_root -- root of the project
# crate_root -- root of the crate

# certora endpoint
export CERTORA_SOLANA_CLI := env_var_or_default("CERTORA_SOLANA_CLI", "certoraSolanaProver")

# certora env files
certora-scripts := project_root / "certora/envs"

inliner_cfg   := certora-scripts  / "inlining.txt"
summaries_cfg := certora-scripts  / "summaries.txt"


# recipes for running certoraSolanaProver
confs_justfile := crate_root / "src/certora/confs/justfile"

	
certora RULE: 
	just -f {{ confs_justfile }} certora {{ RULE }}

certora-conf CONF *OPTS: 
	cd {{ parent_dir(CONF) }} && ${CERTORA_SOLANA_CLI} {{ file_name(CONF) }} {{ OPTS }}


deep-clean: clean
	just -f {{ confs_justfile }} clean




