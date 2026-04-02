.PHONY: build, clean, fmt, lint, deploy, verify, release, new-keypair, check-size, demo-init, demo-create, demo-vote, demo-execute

# ---- Config ----
PROGRAM_NAME := hello_dao
TARGET_DIR   := ./target/deploy
PROGRAM_SO   := $(TARGET_DIR)/$(PROGRAM_NAME).so
KEYPAIR_JSON := $(TARGET_DIR)/$(PROGRAM_NAME)-keypair.json

# ---- Colors ----

GREEN := \033[0;32m
CYAN := \033[0;36m
YELLOW := \033[1;33m
RED := \033[0;31m
RESET := \033[0m

# ---- Help (Default Target) ----
help:
	@echo "$(BOLD)Hello DAO - Developer Control Center$(RESET)"
	@echo "Usage: make $(CYAN)<target>$(RESET)"
	@echo ""
	@echo "$(BOLD)Build & Quality:$(RESET)"
	@echo "  $(CYAN)build$(RESET)         Compile SBF program"
	@echo "  $(CYAN)clean$(RESET)         Remove build artifacts"
	@echo "  $(CYAN)fmt$(RESET)           Format code with rustfmt"
	@echo "  $(CYAN)lint$(RESET)          Run clippy diagnostics"
	@echo ""
	@echo "$(BOLD)Deployment:$(RESET)"
	@echo "  $(CYAN)new-keypair$(RESET)   Generate fresh program ID"
	@echo "  $(CYAN)check-size$(RESET)    Calculate rent-exempt costs"
	@echo "  $(CYAN)deploy$(RESET)        Deploy to cluster (requires $(YELLOW)AUTH=path$(RESET))"
	@echo "  $(CYAN)release$(RESET)       Full Build -> Deploy -> Verify cycle"
	@echo ""
	@echo "$(BOLD)Simulations:$(RESET)"
	@echo "  $(CYAN)demo-init$(RESET)     Initialize DAO & Vault"
	@echo "  $(CYAN)demo-create$(RESET)   Create proposal"
	@echo "  $(CYAN)demo-vote$(RESET)     Cast votes"
	@echo "  $(CYAN)demo-execute$(RESET)  Execute payout"

# ---- Build & Maintenance ----

build:
	@clear
	@echo "$(CYAN)🔧 [BUILD] Compiling $(PROGRAM_NAME)...$(RESET)"
	@cargo build-sbf

clean:
	@echo "$(YELLOW)🧹 [CLEAN] Removing build artifacts...$(RESET)"
	@cargo clean

fmt:
	@clear
	@echo "$(GREEN)🎨 [FMT] Formatting codebase...$(RESET)"
	@cargo fmt

lint:
	@clear
	@echo "$(YELLOW)🧹 [LINT] Running Clippy linter...$(RESET)"
	@cargo clippy 

# ---- Deploy & Authority ----

deploy:
	@if [ -z "$(AUTH)" ]; then \
		echo "$(RED)🔴 [DEPLOY] Missing AUTH argument$(RESET)"; \
		echo "   Usage: make deploy AUTH=~/.config/solana/id.json"; \
		exit 1; \
	fi
	@echo "$(CYAN)🚢 [DEPLOY] Deploying program to Solana...$(RESET)"
	@solana program deploy \
		--program-id $(KEYPAIR_JSON) \
		--upgrade-authority $(AUTH) \
		$(PROGRAM_SO)
	@echo "$(GREEN)🟢 [DEPLOY] Deployment complete.$(RESET)"

verify:
	@echo ""
	@echo "$(CYAN)🔎 [VERIFY] Checking deployed program info...$(RESET)"
	@solana program show $(KEYPAIR_JSON)

release:
	@$(MAKE) build
	@$(MAKE) deploy
	@$(MAKE) verify

# ---- Keypair Management ----

new-keypair:
	@mkdir -p $(TARGET_DIR)
	@if [ -f $(KEYPAIR_JSON) ]; then \
		echo "$(YELLOW)⚠️ [KEYPAIR] Existing keypair found. Backing up...$(RESET)"; \
		mv $(KEYPAIR_JSON) $(KEYPAIR_JSON).$(shell date +%s).bak; \
	fi
	@solana-keygen new --no-passphrase -o $(KEYPAIR_JSON)
	@echo "$(GREEN)✨ [KEYPAIR] New Program ID generated:$(RESET)"
	@solana address -k $(KEYPAIR_JSON)
	@echo "$(YELLOW)👉 [ACTION] Copy the address above and update 'declare_id!' in your lib.rs$(RESET)"

# ---- Build Stats ----

check-size:
	@echo "$(GREEN)📐 [MEASURE] Checking program size...$(RESET)"
	@du -sh $(PROGRAM_SO)
	@SIZE=$$(stat -f%z target/deploy/hello_dao.so 2>/dev/null || stat -c%s $(PROGRAM_SO)); \
	echo "$(YELLOW)💰 [RENT] Estimated rent exempt balance for $$SIZE bytes:$(RESET)"; \
	solana rent $$SIZE
	
# ---- Demos ----

demo-init:
	@npm run simulate init

demo-create:
	@npm run simulate create

demo-vote:
	@npm run simulate vote

demo-execute:
	@npm run simulate execute
