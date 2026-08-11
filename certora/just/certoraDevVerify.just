# Settings for devs that want to run the jar locally
# Importing justfile must define the following variables
# sbf_file -- location of .so file to verify

import "build.just"
import "certoraSolanaProver.just"

# java executable
export JAVA := env_var_or_default("JAVA", "java")
# optimization level, 0 by default
export TAC_OPT_LEVEL := env_var_or_default("TAC_OPT_LEVEL", "0")
# sanity check level: none, basic, advanced (none by default)
export SANITY_CHECKS := env_var_or_default("SANITY_CHECKS", "none")
# location of prover jar
emv_jar := "$CERTORA/emv.jar"

# Usage: verify name_of_rule extra_options
verify RULE *OPTS: build-sbf 
		mkdir -p certora_out
		cd certora_out && ${JAVA} -ea -Xmx8g \
		-Dtopic.spec -Dlevel.sbf=${CERTORA_VERBOSE:-info} \
		-Dverbose.times -Dcvt.simple.parallel -Djava.awt.headless=true \
		-jar {{ emv_jar }} \
		-deleteSMTFile false -graphDrawLimit 2000 \
		{{ sbf_file }} \
		-unsatCoresForAllAsserts true \
		-solanaInlining {{ inliner_cfg }} \
		-solanaSummaries {{ summaries_cfg }} \
		-solanaOptimisticJoin true \
		-solanaOptimisticOverlaps true \
		-solanaOptimisticMemcpyPromotion true \
		-solanaOptimisticMemcmp true \
		-solanaOptimisticNoMemmove true \
		-solanaAggressiveGlobalDetection true \
		-solanaRemoveCFGDiamonds true \
		-solanaTACMathInt true \
		-solanaStackSize 8192 \
		-solanaPrintDevMsg true \
		-solanaPrintAnalyzedToDot \
		-solanaTACOptimize ${TAC_OPT_LEVEL} \
		-ruleSanityChecks ${SANITY_CHECKS} \
		-rule {{ RULE }} \
		{{ OPTS }} 


