# Frequently Asked Questions (FAQ)

## Overview

This document answers the most commonly asked questions about the Soroban Registry. Questions are organized by category for easy navigation.

> **Can't find your answer?** Check the [Troubleshooting Guide](./TROUBLESHOOTING.md) for step-by-step solutions to common issues, or [open an issue](https://github.com/ALIPHATICHYD/Soroban-Registry/issues).

---

## Table of Contents

- [General Questions](#general-questions)
- [Platform and Network Questions](#platform-and-network-questions)
- [Contract Verification Questions](#contract-verification-questions)
- [Publishing and Versioning Questions](#publishing-and-versioning-questions)
- [Integration and API Questions](#integration-and-api-questions)
- [Cost and Limits Questions](#cost-and-limits-questions)
- [Security Questions](#security-questions)
- [Business and Support Questions](#business-and-support-questions)

---

## General Questions

### Q1: What is the Soroban Registry?

The Soroban Registry is a comprehensive platform for discovering, publishing, and verifying Soroban smart contracts on the Stellar network. Think of it as **npm for Soroban contracts** — a centralized place where developers can share, find, and verify smart contracts.

**Key capabilities:**
- **Discover** contracts by name, category, or functionality
- **Publish** your contracts with version management
- **Verify** that on-chain bytecode matches published source code
- **Analyze** contract usage with built-in analytics

---

### Q2: Why should I use the Soroban Registry?

- **Trust** — Verified source code means users can audit what they're interacting with
- **Discoverability** — Find existing contracts instead of building from scratch
- **Version management** — Track contract updates and changelogs
- **Community** — Share your work and build on others' contributions
- **Analytics** — Understand how your contracts are used

---

### Q3: How do I get started?

**Quick start with Docker (recommended):**
```bash
git clone https://github.com/ALIPHATICHYD/Soroban-Registry.git
cd Soroban-Registry
cp .env.example .env
docker-compose up -d
```

- **Web UI:** http://localhost:3000
- **API:** http://localhost:3001

For manual setup, see the [README](../README.md#option-2-manual-setup).

---

### Q4: What are the system requirements?

| Component | Minimum Version |
|-----------|----------------|
| Rust | 1.75+ |
| Node.js | 20+ |
| PostgreSQL | 16+ |
| Docker | 24+ (optional) |

**Hardware recommendations:**
- **Development:** 4 GB RAM, 2 CPU cores, 10 GB disk
- **Production:** 8 GB RAM, 4 CPU cores, 50 GB disk

---

### Q5: Is the Soroban Registry open source?

Yes. The Soroban Registry is licensed under the MIT License. You can view the full license in the [LICENSE](../LICENSE) file. Contributions are welcome — see [CONTRIBUTING.md](../CONTRIBUTING.md).

---

### Q6: What programming languages does the registry support?

- **Backend:** Rust (Axum framework)
- **Frontend:** TypeScript (Next.js)
- **CLI:** Rust
- **Smart contracts:** Rust (via Soroban SDK)
- **API clients:** Any language that supports HTTP/JSON (examples provided in Python, JavaScript/TypeScript, and Rust)

---

### Q7: How is this different from deploying contracts directly?

Deploying a contract puts bytecode on-chain, but doesn't make it discoverable or verifiable by others. The Soroban Registry adds:

| Feature | Direct Deploy | With Registry |
|---------|:------------:|:-------------:|
| On-chain deployment | ✅ | ✅ |
| Source code visibility | ❌ | ✅ |
| Source verification | ❌ | ✅ |
| Search and discovery | ❌ | ✅ |
| Version history | ❌ | ✅ |
| Usage analytics | ❌ | ✅ |
| Publisher profiles | ❌ | ✅ |

---

## Platform and Network Questions

### Q8: Which Stellar networks are supported?

The Soroban Registry supports three networks:

| Network | Purpose | RPC Endpoint |
|---------|---------|-------------|
| **Mainnet** | Production contracts | `https://soroban-rpc.mainnet.stellar.gateway.fm` |
| **Testnet** | Testing and development | `https://soroban-testnet.stellar.org` |
| **Futurenet** | Experimental features | `https://rpc-futurenet.stellar.org` |

---

### Q9: What's the difference between Mainnet and Testnet?

| Aspect | Mainnet | Testnet |
|--------|---------|---------|
| **Purpose** | Production, real value | Development, testing |
| **XLM** | Real XLM (has value) | Free test XLM (no value) |
| **Data persistence** | Permanent | May be reset periodically |
| **Use case** | Live applications | Building and testing |

**Recommendation:** Always deploy and test on Testnet first, then publish to Mainnet when ready.

---

### Q10: Can I use the registry with a private/local network?

Yes. Configure a custom RPC endpoint in your environment:

```bash
# .env
STELLAR_RPC_URL=http://localhost:8000/soroban/rpc
STELLAR_NETWORK=standalone
```

This is useful for local development with `stellar quickstart` or custom Stellar Core instances.

---

### Q11: How do I switch between networks?

**CLI:**
```bash
soroban-registry config set network testnet
# or
soroban-registry search "token" --network mainnet
```

**API:**
```bash
curl "http://localhost:3001/api/contracts?network=testnet"
```

**Frontend:** Use the network selector dropdown in the top navigation bar.

---

### Q12: Are contracts on Testnet visible on Mainnet (or vice versa)?

No. Each network is completely separate. A contract published on Testnet will not appear when browsing Mainnet, and vice versa. You must publish and verify on each network independently.

---

## Contract Verification Questions

### Q13: What is contract verification?

Contract verification is the process of proving that published source code compiles to the exact same bytecode that is deployed on-chain. This allows anyone to:

1. **Read the source code** of a deployed contract
2. **Audit the logic** before interacting with it
3. **Trust** that the on-chain contract does what the source says

---

### Q14: How does verification work?

```
1. You submit source code + contract ID + network
                    │
2. Registry compiles source code in a controlled environment
                    │
3. Compiled bytecode hash is compared to on-chain bytecode hash
                    │
4. If hashes match ──▶ Contract is marked as VERIFIED ✅
   If hashes differ ──▶ Verification fails with details ❌
```

---

### Q15: Why is my verification failing?

The most common causes are:

1. **Wrong compiler version** — Pin your `soroban-sdk` to the exact version used during deployment: `soroban-sdk = "=21.0.0"`
2. **Missing `Cargo.lock`** — Always include it in your source
3. **Wrong optimization level** — Most Soroban contracts use `opt-level = "z"`
4. **Dependency version mismatch** — Use exact versions with `=` prefix

For detailed troubleshooting, see the [Verification Troubleshooting Guide](./VERIFICATION_TROUBLESHOOTING.md).

---

### Q16: Do I need to verify my contract?

Verification is **optional but strongly recommended**. Benefits include:

- Increased user trust and adoption
- Visibility in "Verified Contracts" listings
- A verified badge on your contract page
- Better discoverability in search results

Unverified contracts are still listed but marked as "Unverified."

---

### Q17: How long does verification take?

| Contract Complexity | Typical Time |
|-------------------|-------------|
| Simple (few dependencies) | 30–60 seconds |
| Medium (standard DeFi) | 1–3 minutes |
| Complex (many dependencies) | 3–5 minutes |

If verification takes longer than 5 minutes, check the [Troubleshooting Guide](./TROUBLESHOOTING.md#issue-42-verification-stuck-in-pending).

---

### Q18: Can I verify a contract I didn't deploy?

Yes, anyone can submit source code for verification. You don't need to be the deployer or publisher. This promotes community auditing and transparency.

---

### Q19: What happens if I update my contract?

Each contract version needs separate verification. When you deploy a new version:

1. Deploy the new contract on-chain
2. Publish the new version to the registry
3. Submit the new source code for verification

The registry maintains verification history for all versions.

---

## Publishing and Versioning Questions

### Q20: How do I publish a contract?

**Using the CLI:**
```bash
soroban-registry publish \
  --name "My Token Contract" \
  --description "An ERC-20-like token for Stellar" \
  --category "defi" \
  --contract-path ./my-contract \
  --network testnet
```

**Using the API:**
```bash
curl -X POST http://localhost:3001/api/contracts \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Token Contract",
    "description": "An ERC-20-like token for Stellar",
    "category": "defi",
    "network": "testnet",
    "contract_id": "CDLZFC3..."
  }'
```

**Using the Web UI:** Navigate to "Publish" in the top menu and follow the guided form.

---

### Q21: What categories can I publish to?

Available categories include:

| Category | Description |
|----------|-------------|
| `defi` | Decentralized finance (tokens, DEX, lending) |
| `nft` | Non-fungible tokens and collectibles |
| `dao` | Governance and DAOs |
| `gaming` | Gaming and metaverse |
| `identity` | Identity and credentials |
| `oracle` | Oracles and data feeds |
| `bridge` | Cross-chain bridges |
| `utility` | General-purpose utilities |
| `examples` | Example and tutorial contracts |

---

### Q22: How does versioning work?

The registry follows [Semantic Versioning](https://semver.org/) (SemVer):

```
MAJOR.MINOR.PATCH
  │     │     │
  │     │     └── Bug fixes (backward compatible)
  │     └──────── New features (backward compatible)
  └────────────── Breaking changes
```

**Example:**
```bash
# Initial release
soroban-registry publish --version 1.0.0

# Bug fix
soroban-registry publish --version 1.0.1

# New feature
soroban-registry publish --version 1.1.0

# Breaking change
soroban-registry publish --version 2.0.0
```

---

### Q23: Can I unpublish or delete a contract?

Published contracts **cannot be deleted** from the registry to ensure stability for users who depend on them. However, you can:

1. **Deprecate** a contract version — marks it as no longer recommended
2. **Yank** a specific version — prevents new users from depending on it (existing users unaffected)
3. **Transfer ownership** — if you're no longer maintaining it

---

### Q24: Can multiple people publish to the same contract?

A contract is associated with a single publisher account. To enable team publishing:

1. Create a shared publisher profile
2. Add team members as collaborators
3. Each member can publish new versions under the shared profile

---

### Q25: What metadata should I include when publishing?

**Required:**
- Name
- Description
- Contract ID
- Network

**Recommended:**
- Category/tags
- README or documentation link
- License
- Repository URL
- Changelog for each version

Good metadata improves discoverability and helps users understand your contract.

---

## Integration and API Questions

### Q26: How do I authenticate with the API?

Include your API key in the `Authorization` header:

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  http://localhost:3001/api/contracts
```

**To obtain an API key:**
1. Create a publisher profile via the web UI or API
2. Generate an API key from your profile settings
3. Store the key securely — it won't be shown again

---

### Q27: What API endpoints are available?

**Core Endpoints:**

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/contracts` | List/search contracts |
| `GET` | `/api/contracts/:id` | Get contract details |
| `POST` | `/api/contracts` | Publish a contract |
| `GET` | `/api/contracts/:id/versions` | Get version history |
| `POST` | `/api/contracts/verify` | Verify a contract |
| `GET` | `/api/publishers/:id` | Get publisher details |
| `GET` | `/api/stats` | Registry statistics |
| `GET` | `/health` | Health check |

For advanced features (filtering, sorting, batch operations), see [API Advanced Features](./API_ADVANCED_FEATURES.md).

---

### Q28: Is there a rate limit on API requests?

Yes. Rate limits depend on your tier:

| Tier | Requests/Minute | Requests/Day |
|------|----------------|-------------|
| Anonymous | 30 | 1,000 |
| Authenticated | 100 | 10,000 |
| Publisher | 300 | 50,000 |

Rate limit headers are included in every response:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 42
```

For full details, see [API Rate Limiting](./API_RATE_LIMITING.md).

---

### Q29: Can I use the API from a frontend application?

Yes, but be aware of:

1. **CORS** — The API allows requests from configured origins. Set `FRONTEND_URL` in your API configuration.
2. **API key security** — Never expose your API key in client-side code. Use a backend proxy for authenticated requests.
3. **Rate limits** — Frontend applications should implement caching and debouncing to stay within limits.

---

### Q30: Is there a WebSocket API for real-time updates?

Yes, the registry provides WebSocket connections for:

- New contract publications
- Verification status changes
- Indexer progress updates

```javascript
const ws = new WebSocket('ws://localhost:3001/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event:', data.type, data.payload);
};
```

---

### Q31: How do I handle API errors?

All errors follow a consistent JSON format:

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable description",
  "code": 400,
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Best practices:**
1. Always check the HTTP status code
2. Parse the `error` field for programmatic handling
3. Log the `correlation_id` for debugging
4. Implement retry with backoff for 5xx and 429 errors

For the full error code reference, see [Error Codes](./ERROR_CODES.md).

---

### Q32: Can I integrate the registry into my CI/CD pipeline?

Yes. Common CI/CD integrations:

```yaml
# GitHub Actions example
- name: Publish to Soroban Registry
  run: |
    soroban-registry publish \
      --name "${{ env.CONTRACT_NAME }}" \
      --contract-path ./my-contract \
      --network testnet \
      --version "${{ github.ref_name }}"
  env:
    SOROBAN_REGISTRY_API_KEY: ${{ secrets.REGISTRY_API_KEY }}

- name: Verify contract
  run: |
    soroban-registry verify \
      --contract-id "${{ env.CONTRACT_ID }}" \
      --source ./my-contract/src \
      --network testnet
```

---

## Cost and Limits Questions

### Q33: Is the Soroban Registry free to use?

**Free for:**
- Browsing and searching contracts
- Viewing contract source code and verification status
- Using the API within anonymous rate limits
- Publishing contracts (no per-contract fees)

**Costs to be aware of:**
- Stellar network transaction fees apply when deploying contracts on-chain (not charged by the registry)
- Self-hosted deployments require your own infrastructure
- Higher API rate limits may be available for enterprise users

---

### Q34: Are there limits on contract size?

| Limit | Value |
|-------|-------|
| Maximum WASM binary size | 256 KB (Stellar network limit) |
| Maximum source archive size | 10 MB |
| Maximum number of source files | 500 |
| Maximum file size (individual) | 1 MB |

---

### Q35: How many contracts can I publish?

There is **no hard limit** on the number of contracts you can publish. However:

- Each contract must be unique (no duplicate contract IDs per network)
- Automated mass-publishing may trigger rate limits
- All published contracts must comply with the registry's terms of use

---

### Q36: How many versions can a contract have?

There is no limit on the number of versions per contract. However:

- Each version must have a unique SemVer string
- Versions cannot be overwritten once published
- Older versions remain accessible

---

### Q37: What are the API rate limits and can I increase them?

See [Q28](#q28-is-there-a-rate-limit-on-api-requests) for current limits. If you need higher limits:

1. Authenticate your requests (increases from 30 to 100 req/min)
2. Register as a publisher (increases to 300 req/min)
3. For enterprise needs, contact the maintainers via GitHub Issues

---

## Security Questions

### Q38: How should I handle my API keys?

- **Never** commit API keys to source control
- Use environment variables or secret management tools
- Rotate keys regularly
- Use separate keys for development and production

```bash
# Good: environment variable
export SOROBAN_REGISTRY_API_KEY=your-key-here

# Bad: hardcoded in source
API_KEY = "your-key-here"  # DON'T DO THIS
```

For more security best practices, see [Security](./SECURITY.md).

---

### Q39: Is the verification process tamper-proof?

The verification process includes multiple safeguards:

1. **Deterministic builds** — Same source always produces same bytecode
2. **Controlled environment** — Compilation happens in isolated containers
3. **Hash comparison** — SHA-256 hash of compiled bytecode vs on-chain bytecode
4. **Immutable records** — Verification results cannot be altered after completion
5. **Audit trail** — All verification attempts are logged

---

### Q40: What if I find a security vulnerability?

**Do NOT open a public issue.** Instead:

1. Email the maintainers directly (see [SECURITY.md](./SECURITY.md) for contact information)
2. Include a clear description of the vulnerability
3. Provide steps to reproduce if possible
4. Allow reasonable time for a fix before public disclosure

---

### Q41: Can someone publish malicious contracts?

The registry does not review or audit contract logic. Verification only confirms that source code matches on-chain bytecode — it does **not** guarantee the code is safe or bug-free.

**Users should always:**
- Read and understand contract source code before interacting
- Check the publisher's reputation and history
- Start with small transactions on Testnet
- Look for independent audits

---

## Business and Support Questions

### Q42: How do I report a bug?

1. Check the [Troubleshooting Guide](./TROUBLESHOOTING.md) first
2. Search [existing issues](https://github.com/ALIPHATICHYD/Soroban-Registry/issues) to avoid duplicates
3. Open a new issue with:
   - Steps to reproduce
   - Expected vs actual behavior
   - Error messages and `correlation_id` values
   - Environment details (OS, versions)
   - Relevant logs (redact sensitive data)

---

### Q43: How do I request a feature?

Open a GitHub issue using the **Feature Request** template. Include:

- A clear description of the feature
- The problem it solves
- Example use cases
- Any proposed implementation ideas

---

### Q44: How can I contribute?

Contributions are welcome! To get started:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes and add tests
4. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

---

### Q45: Where can I get help?

| Channel | Best For |
|---------|----------|
| [GitHub Issues](https://github.com/ALIPHATICHYD/Soroban-Registry/issues) | Bug reports, feature requests |
| [Stellar Community Forum](https://community.stellar.org) | General questions, discussions |
| [Stellar Discord](https://discord.gg/stellar) | Real-time chat, quick questions |
| [Troubleshooting Guide](./TROUBLESHOOTING.md) | Self-service problem resolution |
| This FAQ | Common questions and answers |

---

### Q46: Is there an SLA (Service Level Agreement)?

For the public hosted registry:
- **Uptime target:** 99.5%
- **Planned maintenance:** Announced 24 hours in advance
- **Incident response:** See [Incident Response](./INCIDENT_RESPONSE.md)

For self-hosted deployments, SLA depends on your own infrastructure.

---

### Q47: Can I self-host the registry?

Yes. The Soroban Registry is fully open source and can be self-hosted:

```bash
git clone https://github.com/ALIPHATICHYD/Soroban-Registry.git
cd Soroban-Registry
cp .env.example .env
# Edit .env with your configuration
docker-compose up -d
```

Self-hosting gives you full control over data, configuration, and customization. See the [README](../README.md) for setup instructions.

---

### Q48: How is the registry maintained?

The Soroban Registry is maintained by the community with support from the Stellar ecosystem. Development is coordinated through:

- GitHub Issues for bug tracking and feature requests
- Pull requests for code contributions
- Community discussions for design decisions
- Regular releases with changelogs

---

### Q49: What's on the roadmap?

The project roadmap is tracked through GitHub Issues and Milestones. Planned features and improvements are labeled and prioritized. Check the [Issues page](https://github.com/ALIPHATICHYD/Soroban-Registry/issues) for current priorities.

---

### Q50: How do I stay updated on changes?

- **Watch** the GitHub repository for release notifications
- **Star** the repository to show support
- Check the [CHANGELOG](../CHANGELOG.md) for version updates
- Follow the Stellar community channels for announcements

---

## Related Documentation

- [Troubleshooting Guide](./TROUBLESHOOTING.md) — Step-by-step solutions to common issues
- [Error Codes Reference](./ERROR_CODES.md) — All API error codes explained
- [Verification Troubleshooting](./VERIFICATION_TROUBLESHOOTING.md) — Verification-specific issues
- [API Rate Limiting](./API_RATE_LIMITING.md) — Rate limit details and tiers
- [API Advanced Features](./API_ADVANCED_FEATURES.md) — Advanced API usage
- [Security](./SECURITY.md) — Security best practices and vulnerability reporting
- [Observability](./OBSERVABILITY.md) — Monitoring and metrics setup
- [Incident Response](./INCIDENT_RESPONSE.md) — Incident handling procedures
- [Disaster Recovery Plan](./DISASTER_RECOVERY_PLAN.md) — Recovery procedures

## Support

For additional help:
- GitHub Issues: https://github.com/ALIPHATICHYD/Soroban-Registry/issues
- Community Forum: https://community.stellar.org
- Stellar Discord: https://discord.gg/stellar
