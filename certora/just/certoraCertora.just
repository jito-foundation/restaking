# Just file to be placed next to conf files


set dotenv-load

export CERTORA_SOLANA_CLI := env_var_or_default("CERTORA_SOLANA_CLI", "certoraSolanaProver")


certora RULE: 
    ${CERTORA_SOLANA_CLI} run.conf --rule {{ RULE }} --msg {{ RULE }}

clean:
    rm -rf .certora_internal