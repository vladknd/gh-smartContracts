# NFID Integration Guide

> **Last Updated**: January 2026  
> **Purpose**: Replace Internet Identity with NFID for authentication

---

## Table of Contents

1. [Why NFID Instead of Internet Identity?](#why-nfid-instead-of-internet-identity)
2. [Key Differences](#key-differences)
3. [Integration Options](#integration-options)
4. [Option A: NFID IdentityKit (Recommended)](#option-a-nfid-identitykit-recommended)
5. [Option B: Direct AuthClient with NFID](#option-b-direct-authclient-with-nfid)
6. [Principal Behavior](#principal-behavior)
7. [Migration Considerations](#migration-considerations)
8. [Backend Compatibility](#backend-compatibility)

---

## Why NFID Instead of Internet Identity?

| Feature | Internet Identity | NFID |
|---------|------------------|------|
| **Login Methods** | WebAuthn only (passkeys) | Email, Google, passkeys |
| **User Experience** | Confusing for non-crypto users | Familiar (like Google auth) |
| **Account Recovery** | Seed phrase / recovery device | Email recovery |
| **Principal per Origin** | ⚠️ **Different Principal per domain** | ✅ **Same Principal everywhere** |
| **Domain Change Risk** | Users lose access to data | No impact |
| **Multiple Devices** | Add each device manually | Auto-sync via email |
| **Asset/NFT Display** | No | Yes |

### The Critical Difference: Principal Derivation

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    INTERNET IDENTITY vs NFID                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   INTERNET IDENTITY:                                                            │
│   User logs into https://myapp.com → Principal: "aaaaa-bbbbb-ccccc"            │
│   User logs into https://staging.myapp.com → Principal: "xxxxx-yyyyy-zzzzz"    │
│   ⚠️ DIFFERENT PRINCIPALS! Data is tied to domain.                             │
│                                                                                 │
│   NFID:                                                                         │
│   User logs into https://myapp.com → Principal: "aaaaa-bbbbb-ccccc"            │
│   User logs into https://staging.myapp.com → Principal: "aaaaa-bbbbb-ccccc"    │
│   ✅ SAME PRINCIPAL! Data follows the user.                                    │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Key Differences

### How It Works

```
NFID Authentication Flow:
─────────────────────────

1. User clicks "Connect Wallet" in your app
2. NFID modal opens with options:
   ├── Email (enter email, receive code)
   ├── Google (OAuth)
   └── Passkey (WebAuthn, like II)
3. User authenticates
4. NFID returns delegation identity (same format as II)
5. Your app receives Principal
6. ✅ Same Principal regardless of which frontend domain!
```

---

## Integration Options

You have two options for integrating NFID:

| Option | Library | Complexity | Features |
|--------|---------|------------|----------|
| **A** | `@nfid/identitykit` | Low | Pre-built UI, wallet selector, multi-wallet support |
| **B** | `@dfinity/auth-client` | Medium | Custom UI, just change provider URL |

---

## Option A: NFID IdentityKit (Recommended)

This is the easiest approach - NFID provides a complete wallet connection kit.

### Step 1: Install Dependencies

```bash
# Install IdentityKit
npm install @nfid/identitykit

# Install required peer dependencies
npm install @dfinity/ledger-icp @dfinity/identity @dfinity/agent \
            @dfinity/candid @dfinity/principal @dfinity/utils @dfinity/auth-client
```

### Step 2: Import Styles

In your main entry file (e.g., `main.tsx` or `App.tsx`):

```typescript
// Import NFID IdentityKit styles
import "@nfid/identitykit/react/styles.css"
```

### Step 3: Wrap Your App with Provider

```typescript
// App.tsx
import { IdentityKitProvider } from "@nfid/identitykit/react"

const App = () => {
  return (
    <IdentityKitProvider>
      <YourApp />
    </IdentityKitProvider>
  )
}

export default App
```

### Step 4: Add Connect Button

```typescript
// ConnectButton.tsx
import { ConnectWallet } from "@nfid/identitykit/react"

export const ConnectButton = () => {
  return <ConnectWallet />
}
```

### Step 5: Use the Identity

```typescript
// useAuth.tsx
import { useIdentityKit } from "@nfid/identitykit/react"
import { Actor, HttpAgent } from "@dfinity/agent"

export const useAuth = () => {
  const { identity, isConnected, principal } = useIdentityKit()
  
  const createActor = async (canisterId: string, idlFactory: any) => {
    if (!identity) throw new Error("Not authenticated")
    
    const agent = new HttpAgent({ identity })
    // Only for local development:
    // await agent.fetchRootKey()
    
    return Actor.createActor(idlFactory, {
      agent,
      canisterId,
    })
  }
  
  return {
    identity,
    isConnected,
    principal,
    createActor,
  }
}
```

### Step 6: Full Integration Example

```typescript
// GHCClient.tsx
import { useIdentityKit } from "@nfid/identitykit/react"
import { Actor, HttpAgent } from "@dfinity/agent"

// Import your canister IDLs
import { idlFactory as userProfileIdl } from "./declarations/user_profile"
import { idlFactory as stakingHubIdl } from "./declarations/staking_hub"
import { idlFactory as treasuryIdl } from "./declarations/treasury_canister"
import { idlFactory as governanceIdl } from "./declarations/governance_canister"
import { idlFactory as ledgerIdl } from "./declarations/ghc_ledger"

import { CANISTER_IDS } from "./canister-ids"

export const useGHCClient = () => {
  const { identity, isConnected, principal } = useIdentityKit()
  
  const getActors = async () => {
    if (!identity) throw new Error("Not authenticated")
    
    const agent = new HttpAgent({ identity })
    // Uncomment for local development:
    // await agent.fetchRootKey()
    
    return {
      userProfile: Actor.createActor(userProfileIdl, {
        agent,
        canisterId: CANISTER_IDS.user_profile,
      }),
      stakingHub: Actor.createActor(stakingHubIdl, {
        agent,
        canisterId: CANISTER_IDS.staking_hub,
      }),
      treasury: Actor.createActor(treasuryIdl, {
        agent,
        canisterId: CANISTER_IDS.treasury_canister,
      }),
      governance: Actor.createActor(governanceIdl, {
        agent,
        canisterId: CANISTER_IDS.governance_canister,
      }),
      ledger: Actor.createActor(ledgerIdl, {
        agent,
        canisterId: CANISTER_IDS.ghc_ledger,
      }),
    }
  }
  
  return {
    isConnected,
    principal,
    getActors,
  }
}

// Usage in a component:
const Dashboard = () => {
  const { isConnected, principal, getActors } = useGHCClient()
  const [profile, setProfile] = useState(null)
  
  useEffect(() => {
    if (isConnected && principal) {
      loadProfile()
    }
  }, [isConnected, principal])
  
  const loadProfile = async () => {
    const { userProfile } = await getActors()
    const result = await userProfile.get_profile(principal)
    if (result.length > 0) {
      setProfile(result[0])
    }
  }
  
  if (!isConnected) {
    return <ConnectWallet />
  }
  
  return (
    <div>
      <p>Welcome, {profile?.name || principal?.toString()}</p>
    </div>
  )
}
```

---

## Option B: Direct AuthClient with NFID

If you want to keep using `@dfinity/auth-client` with custom UI, just change the identity provider URL.

### The Only Change Needed

```typescript
// OLD: Internet Identity
import { AuthClient } from "@dfinity/auth-client"

const authClient = await AuthClient.create()
await authClient.login({
  identityProvider: "https://identity.ic0.app",  // Internet Identity
  onSuccess: () => { /* authenticated */ },
})
```

```typescript
// NEW: NFID (just change the URL!)
import { AuthClient } from "@dfinity/auth-client"

const authClient = await AuthClient.create()
await authClient.login({
  identityProvider: "https://nfid.one/authenticate",  // ← NFID!
  onSuccess: () => { /* authenticated */ },
})
```

### Complete AuthClient Example with NFID

```typescript
// auth.ts
import { AuthClient } from "@dfinity/auth-client"
import { Actor, HttpAgent, Identity } from "@dfinity/agent"
import { Principal } from "@dfinity/principal"

// Configuration
const NFID_PROVIDER_URL = "https://nfid.one/authenticate"

// For advanced configuration (optional):
// https://nfid.one/authenticate?applicationName=GreenHeroCoin&applicationLogo=https://yoursite.com/logo.png

class AuthService {
  private authClient: AuthClient | null = null
  private identity: Identity | null = null
  
  async init(): Promise<boolean> {
    this.authClient = await AuthClient.create()
    
    // Check if already authenticated
    if (await this.authClient.isAuthenticated()) {
      this.identity = this.authClient.getIdentity()
      return true
    }
    return false
  }
  
  async login(): Promise<Principal> {
    if (!this.authClient) {
      await this.init()
    }
    
    return new Promise((resolve, reject) => {
      this.authClient!.login({
        identityProvider: NFID_PROVIDER_URL,
        
        // Optional: Specify max time for delegation
        maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000), // 7 days
        
        onSuccess: () => {
          this.identity = this.authClient!.getIdentity()
          resolve(this.identity.getPrincipal())
        },
        
        onError: (error) => {
          reject(new Error(error || "Authentication failed"))
        },
      })
    })
  }
  
  async logout(): Promise<void> {
    await this.authClient?.logout()
    this.identity = null
  }
  
  getIdentity(): Identity | null {
    return this.identity
  }
  
  getPrincipal(): Principal | null {
    return this.identity?.getPrincipal() || null
  }
  
  isAuthenticated(): boolean {
    return this.identity !== null && !this.identity.getPrincipal().isAnonymous()
  }
  
  async createAgent(): Promise<HttpAgent> {
    if (!this.identity) {
      throw new Error("Not authenticated")
    }
    
    const agent = new HttpAgent({ identity: this.identity })
    
    // Only for local development:
    // await agent.fetchRootKey()
    
    return agent
  }
}

export const authService = new AuthService()

// Usage:
// await authService.init()
// if (!authService.isAuthenticated()) {
//   await authService.login()
// }
// const agent = await authService.createAgent()
```

---

## Principal Behavior

### NFID vs Internet Identity

| Scenario | Internet Identity | NFID |
|----------|------------------|------|
| Login from `https://app.com` | Principal A | Principal X |
| Login from `https://staging.app.com` | Principal B (different!) | Principal X (same!) |
| Login from `https://newdomain.com` | Principal C (different!) | Principal X (same!) |
| Change email | N/A | Principal X (same!) |
| Use different browser | Same Principal | Same Principal |
| Use different device | Same Principal | Same Principal |

### Why This Matters for GHC

Your canisters identify users by Principal:

```rust
// In user_profile canister
#[update]
async fn register_user(args: UserProfileUpdate) -> Result<(), String> {
    let user = ic_cdk::caller();  // This is the Principal
    
    // With II: user changes domain → new Principal → can't find their profile!
    // With NFID: user changes domain → same Principal → profile found!
    
    USER_PROFILES.with(|p| p.borrow_mut().insert(user, new_profile));
}
```

---

## Migration Considerations

### New Users (Fresh Start)

If you haven't launched yet or have few users, simply use NFID from the start:
- ✅ No migration needed
- ✅ All users will have consistent Principals

### Existing Users (Already Using II)

If you have existing users with Internet Identity:

**Option 1: Support Both (Recommended)**

```typescript
// Let users choose their provider
const login = async (provider: "ii" | "nfid") => {
  const authClient = await AuthClient.create()
  
  const providerUrl = provider === "nfid" 
    ? "https://nfid.one/authenticate"
    : "https://identity.ic0.app"
  
  await authClient.login({
    identityProvider: providerUrl,
    onSuccess: () => { /* ... */ },
  })
}
```

**Option 2: Implement Account Linking**

Allow users to link their II Principal with their NFID Principal:

```rust
// In user_profile canister - add this method
#[derive(CandidType, Deserialize)]
struct LinkIdentityRequest {
    target_principal: Principal,
    signature: Vec<u8>,  // Prove ownership of target
}

// Map: AltPrincipal -> MainPrincipal
static PRINCIPAL_ALIASES: RefCell<StableBTreeMap<Principal, Principal, Memory>> = ...;

#[update]
fn link_identity(request: LinkIdentityRequest) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Verify the caller owns the target_principal
    // (This requires the user to sign a message with both identities)
    
    // Store mapping
    PRINCIPAL_ALIASES.with(|a| {
        a.borrow_mut().insert(request.target_principal, caller)
    });
    
    Ok(())
}

// Modify get_profile to check aliases
#[query]
fn get_profile(user: Principal) -> Option<UserProfile> {
    // First check if this principal is an alias
    let resolved_principal = PRINCIPAL_ALIASES.with(|a| {
        a.borrow().get(&user).unwrap_or(user)
    });
    
    USER_PROFILES.with(|p| p.borrow().get(&resolved_principal))
}
```

---

## Backend Compatibility

### ✅ No Backend Changes Required!

Your canisters will work with NFID without any modifications:

| Canister | Change Required | Reason |
|----------|----------------|--------|
| `user_profile` | ❌ None | `ic_cdk::caller()` works the same |
| `staking_hub` | ❌ None | Principal-based auth unchanged |
| `governance_canister` | ❌ None | Board members identified by Principal |
| `treasury_canister` | ❌ None | Governance canister Principal unchanged |
| `learning_engine` | ❌ None | Stateless, no auth |
| `founder_vesting` | ❌ None | Founder Principals remain fixed |

The only thing that matters to your canisters is the **Principal**. Both II and NFID provide a valid Principal - the canister doesn't know (or care) which provider was used.

---

## Summary: Quick Start

### For New Project

```bash
# 1. Install
npm install @nfid/identitykit @dfinity/agent @dfinity/principal \
            @dfinity/candid @dfinity/identity @dfinity/auth-client

# 2. Import styles in your main file
import "@nfid/identitykit/react/styles.css"

# 3. Wrap app
<IdentityKitProvider>
  <App />
</IdentityKitProvider>

# 4. Add button
<ConnectWallet />

# 5. Use identity
const { identity, principal } = useIdentityKit()
```

### For Existing II Integration

Just change one line:

```diff
- identityProvider: "https://identity.ic0.app"
+ identityProvider: "https://nfid.one/authenticate"
```

---

## Resources

- [NFID IdentityKit Documentation](https://identitykit.xyz/)
- [NFID IdentityKit GitHub](https://github.com/internet-identity-labs/identitykit)
- [NFID Website](https://nfid.one/)
- [DFINITY Auth Client](https://github.com/dfinity/agent-js/tree/main/packages/auth-client)

---

## Related Documentation

- [Architecture](./ARCHITECTURE.md)
- [Frontend Integration](./FRONTEND_INTEGRATION.md)
- [Identity Recovery Strategies](./IDENTITY_RECOVERY_STRATEGIES.md)
