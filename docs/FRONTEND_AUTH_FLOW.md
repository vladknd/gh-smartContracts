# Frontend Authentication Flow

> **Last Updated**: January 2026  
> **Purpose**: Troubleshoot and properly implement frontend authentication with Internet Identity / NFID

---

## Table of Contents

1. [Common Authentication Issues](#common-authentication-issues)
2. [How Authentication Works](#how-authentication-works)
3. [Proper Login/Logout Flow](#proper-loginlogout-flow)
4. [Debugging Authentication Problems](#debugging-authentication-problems)
5. [Best Practices](#best-practices)

---

## Common Authentication Issues

### Issue: "All users see the same profile"

This is the most common authentication bug. Users log in with different Internet Identity accounts, but the frontend shows the same user profile.

**Root Cause**: The frontend is not properly clearing cached identity/delegation when switching users.

**NOT the cause**: The backend is NOT caching or sharing profiles. The `user_profile` canister correctly isolates data by Principal.

### Verification Steps

1. **Call `whoami` from your frontend** after authentication:
   ```typescript
   const principal = await userProfileActor.whoami();
   console.log("Backend sees me as:", principal.toString());
   ```

2. **Compare with frontend principal**:
   ```typescript
   const frontendPrincipal = identity.getPrincipal();
   console.log("Frontend identity:", frontendPrincipal.toString());
   
   // These should ALWAYS match!
   if (principal.toString() !== frontendPrincipal.toString()) {
     console.error("MISMATCH! Authentication is broken.");
   }
   ```

3. **Use admin debug endpoint** (as controller):
   ```bash
   dfx canister call user_profile admin_list_all_users "(0 : nat32, 50 : nat32)"
   ```
   This lists all registered users with their principals - verify you don't have duplicate registrations.

---

## How Authentication Works

### The Delegation Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                    AUTHENTICATION FLOW                                        │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  1. User clicks "Login" → Opens II/NFID popup                                │
│  2. User authenticates with passkey/email/etc                                 │
│  3. II/NFID creates DELEGATION IDENTITY                                       │
│     └── This is a temporary keypair signed by the master key                 │
│  4. Delegation returned to frontend                                           │
│  5. Frontend stores delegation in localStorage/IndexedDB                      │
│  6. Frontend creates HttpAgent with delegation identity                       │
│  7. All canister calls include the identity, backend gets Principal          │
│                                                                               │
│  CRITICAL: Step 5 is where caching issues occur!                             │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Where Delegations Are Stored

| Library | Storage Location | Key Names |
|---------|------------------|-----------|
| `@dfinity/auth-client` | IndexedDB + localStorage | `ic-identity`, `ic-delegation` |
| `@nfid/identitykit` | IndexedDB | `nfid-*` keys |

### Internet Identity Origin-Specific Principals

**IMPORTANT**: Internet Identity generates **different Principals for different origins**:

```
Same II anchor, different results:
  • https://myapp.ic0.app  → Principal: aaaaa-bbbbb
  • https://staging.ic0.app → Principal: xxxxx-yyyyy  (DIFFERENT!)
  • http://localhost:4943   → Principal: zzzzz-wwwww  (DIFFERENT!)
```

This is by design for privacy, but causes issues when:
- Moving from staging to production
- Changing domain names
- Testing locally vs deployed

**Solution**: Use NFID instead, which provides the same Principal regardless of origin. See [NFID_INTEGRATION_GUIDE.md](./NFID_INTEGRATION_GUIDE.md).

---

## Proper Login/Logout Flow

### Correct Implementation

```typescript
// auth-service.ts
import { AuthClient } from "@dfinity/auth-client";
import { Actor, HttpAgent, Identity } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";

class AuthService {
  private authClient: AuthClient | null = null;
  private identity: Identity | null = null;
  private agent: HttpAgent | null = null;
  
  // ==========================================================================
  // INITIALIZATION
  // ==========================================================================
  
  async init(): Promise<boolean> {
    // Create new AuthClient - this loads any cached delegation
    this.authClient = await AuthClient.create();
    
    // Check if we have a valid (non-expired) delegation
    if (await this.authClient.isAuthenticated()) {
      this.identity = this.authClient.getIdentity();
      
      // IMPORTANT: Verify the identity is not anonymous
      if (!this.identity.getPrincipal().isAnonymous()) {
        await this.createAgent();
        return true;
      }
    }
    
    // No valid authentication
    this.identity = null;
    return false;
  }
  
  // ==========================================================================
  // LOGIN - The Critical Part
  // ==========================================================================
  
  async login(): Promise<Principal> {
    if (!this.authClient) {
      await this.init();
    }
    
    return new Promise((resolve, reject) => {
      this.authClient!.login({
        // For NFID (recommended):
        identityProvider: "https://nfid.one/authenticate",
        
        // Or for Internet Identity:
        // identityProvider: "https://identity.ic0.app",
        
        // Delegation validity (7 days recommended)
        maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000),
        
        onSuccess: async () => {
          // Get the NEW identity from AuthClient
          this.identity = this.authClient!.getIdentity();
          
          // Create a NEW agent with this identity
          await this.createAgent();
          
          // Resolve with the principal
          resolve(this.identity.getPrincipal());
        },
        
        onError: (error) => {
          reject(new Error(error || "Authentication failed"));
        },
      });
    });
  }
  
  // ==========================================================================
  // LOGOUT - Clear Everything!
  // ==========================================================================
  
  async logout(): Promise<void> {
    // 1. Call AuthClient logout (clears its internal state)
    await this.authClient?.logout();
    
    // 2. Clear our references
    this.identity = null;
    this.agent = null;
    
    // 3. Clear any app-specific cached state
    // (Replace with your actual state management)
    window.sessionStorage.clear();
    
    // 4. OPTIONAL: Clear all IndexedDB to be extra safe
    // This is aggressive but ensures no stale delegations
    try {
      const databases = await indexedDB.databases();
      for (const db of databases) {
        if (db.name?.startsWith("ic-") || db.name?.startsWith("nfid-")) {
          indexedDB.deleteDatabase(db.name);
        }
      }
    } catch (e) {
      console.warn("Could not clear IndexedDB:", e);
    }
    
    // 5. Reload the page to ensure fresh state
    window.location.reload();
  }
  
  // ==========================================================================
  // AGENT CREATION
  // ==========================================================================
  
  private async createAgent(): Promise<void> {
    if (!this.identity) {
      throw new Error("No identity available");
    }
    
    this.agent = new HttpAgent({
      identity: this.identity,
      host: process.env.DFX_NETWORK === "ic" 
        ? "https://ic0.app" 
        : "http://localhost:4943",
    });
    
    // Only for local development!
    if (process.env.DFX_NETWORK !== "ic") {
      await this.agent.fetchRootKey();
    }
  }
  
  // ==========================================================================
  // HELPERS
  // ==========================================================================
  
  getIdentity(): Identity | null {
    return this.identity;
  }
  
  getPrincipal(): Principal | null {
    return this.identity?.getPrincipal() || null;
  }
  
  getAgent(): HttpAgent | null {
    return this.agent;
  }
  
  isAuthenticated(): boolean {
    return (
      this.identity !== null && 
      !this.identity.getPrincipal().isAnonymous()
    );
  }
  
  // Create an actor for a specific canister
  createActor<T>(canisterId: string, idlFactory: any): T {
    if (!this.agent) {
      throw new Error("Not authenticated");
    }
    
    return Actor.createActor(idlFactory, {
      agent: this.agent,
      canisterId,
    }) as T;
  }
}

export const authService = new AuthService();
```

### React Integration

```typescript
// AuthContext.tsx
import React, { createContext, useContext, useState, useEffect, useCallback } from "react";
import { Principal } from "@dfinity/principal";
import { authService } from "./auth-service";

interface AuthContextType {
  isAuthenticated: boolean;
  isLoading: boolean;
  principal: Principal | null;
  principalText: string;
  login: () => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [principal, setPrincipal] = useState<Principal | null>(null);
  
  // Initialize on mount
  useEffect(() => {
    const init = async () => {
      try {
        const wasAuthenticated = await authService.init();
        if (wasAuthenticated) {
          setIsAuthenticated(true);
          setPrincipal(authService.getPrincipal());
        }
      } catch (e) {
        console.error("Auth init failed:", e);
      } finally {
        setIsLoading(false);
      }
    };
    
    init();
  }, []);
  
  const login = useCallback(async () => {
    try {
      setIsLoading(true);
      const newPrincipal = await authService.login();
      setPrincipal(newPrincipal);
      setIsAuthenticated(true);
      
      // Debug: Log the principal
      console.log("Logged in as:", newPrincipal.toString());
    } finally {
      setIsLoading(false);
    }
  }, []);
  
  const logout = useCallback(async () => {
    await authService.logout();
    // Page will reload, so these won't matter
    setIsAuthenticated(false);
    setPrincipal(null);
  }, []);
  
  return (
    <AuthContext.Provider
      value={{
        isAuthenticated,
        isLoading,
        principal,
        principalText: principal?.toString() || "",
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within AuthProvider");
  }
  return context;
};
```

---

## Debugging Authentication Problems

### Debug Checklist

1. **Add principal display to UI during development**:
   ```tsx
   // In your dashboard/header component
   const { principal } = useAuth();
   
   return (
     <div style={{ fontSize: "10px", color: "#888" }}>
       Principal: {principal?.toString()}
     </div>
   );
   ```

2. **Verify backend sees correct principal**:
   ```typescript
   // Call whoami endpoint
   const backendPrincipal = await userProfileActor.whoami();
   console.log("Backend sees:", backendPrincipal.toString());
   ```

3. **Check for stale delegations**:
   ```typescript
   // In browser console
   console.log("LocalStorage:", Object.keys(localStorage));
   console.log("SessionStorage:", Object.keys(sessionStorage));
   
   // Check IndexedDB
   const databases = await indexedDB.databases();
   console.log("IndexedDB:", databases);
   ```

4. **Test with different browsers/profiles**:
   - Chrome vs Firefox
   - Chrome Normal vs Chrome Incognito
   - Each should give different II anchors and principals

5. **Use the test script**:
   ```bash
   ./scripts/tests/test_user_profile_auth.sh
   ```
   This verifies the backend is working correctly. If tests pass but frontend fails, the issue is definitely frontend-side.

### Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Not calling `logout()` before new login | Old delegation persists | Always logout first |
| Reusing old HttpAgent | Agent has old identity | Create new agent after login |
| Caching actors globally | Actors use old identity | Recreate actors after login |
| Not clearing React state | UI shows old data | Reset state on logout |
| Using II in local mode | Same principal always | Use different anchor numbers |

---

## Best Practices

### 1. Always Create Fresh Agents After Login

```typescript
// BAD - agent created once, reused forever
const agent = new HttpAgent({ identity });
const actor = Actor.createActor(idl, { agent, canisterId });

// GOOD - agent created fresh after each login
const login = async () => {
  const principal = await authService.login();
  const agent = new HttpAgent({ 
    identity: authService.getIdentity()! 
  });
  // Create actors with new agent
};
```

### 2. Don't Store Actors Globally

```typescript
// BAD - actor stored at module level
const userProfileActor = Actor.createActor(...);

// GOOD - actors created when needed
const getUserProfileActor = () => {
  const agent = authService.getAgent();
  if (!agent) throw new Error("Not authenticated");
  return Actor.createActor(idlFactory, { agent, canisterId });
};
```

### 3. Clear All State on Logout

```typescript
const logout = async () => {
  // 1. Auth client logout
  await authClient.logout();
  
  // 2. Clear app state
  setUser(null);
  setGlobalState(initialState);
  
  // 3. Clear storage
  localStorage.clear();
  sessionStorage.clear();
  
  // 4. Reload
  window.location.reload();
};
```

### 4. Use NFID for Production

Internet Identity creates different principals per origin, which causes issues:
- Staging → Production migration
- Domain changes
- Local testing

NFID provides **consistent principals across all origins**. See [NFID_INTEGRATION_GUIDE.md](./NFID_INTEGRATION_GUIDE.md).

### 5. Display Principal in Development

Always show the current principal during development:

```tsx
{process.env.NODE_ENV === "development" && (
  <div className="debug-info">
    Principal: {principal?.toString()}
  </div>
)}
```

---

## Related Documentation

- [NFID_INTEGRATION_GUIDE.md](./NFID_INTEGRATION_GUIDE.md) - Switching to NFID
- [FRONTEND_INTEGRATION.md](./FRONTEND_INTEGRATION.md) - Complete frontend reference
- [TESTING_GUIDE.md](./TESTING_GUIDE.md) - Testing procedures

---

## Quick Reference: Debug Endpoints

### Backend (user_profile canister)

```bash
# Get your principal as seen by backend
dfx canister call user_profile whoami "()"

# Check if a principal is registered
dfx canister call user_profile is_user_registered "(principal \"YOUR-PRINCIPAL\")"

# Get user profile by principal
dfx canister call user_profile get_profile "(principal \"YOUR-PRINCIPAL\")"

# Admin: List all registered users (controller only)
dfx canister call user_profile admin_list_all_users "(0 : nat32, 50 : nat32)"

# Admin: Get specific user details (controller only)  
dfx canister call user_profile admin_get_user_details "(principal \"USER-PRINCIPAL\")"
```

### Frontend (JavaScript Console)

```javascript
// Check current identity
const auth = await AuthClient.create();
console.log("Authenticated:", await auth.isAuthenticated());
console.log("Principal:", auth.getIdentity().getPrincipal().toString());

// Check storage
console.log("LocalStorage keys:", Object.keys(localStorage));
const dbs = await indexedDB.databases();
console.log("IndexedDB:", dbs.map(d => d.name));

// Clear everything (nuclear option)
localStorage.clear();
sessionStorage.clear();
(await indexedDB.databases()).forEach(db => indexedDB.deleteDatabase(db.name));
location.reload();
```
