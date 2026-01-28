import { Principal } from '@dfinity/principal';
import { HttpAgent, Actor } from '@dfinity/agent';

// This is a boilerplate for a Cloudflare Worker that acts as a Stripe Webhook Bridge
// for the GHC Subscription Canister.

export default {
    async fetch(request, env) {
        if (request.method !== 'POST') {
            return new Response('Method not allowed', { status: 405 });
        }

        const signature = request.headers.get('stripe-signature');
        const body = await request.text();

        // 1. Verify Stripe Webhook Signature (requires stripe-node or similar)
        // For this boilerplate, we'll assume verification is done if secret matches
        // In production, use: stripe.webhooks.constructEvent(body, signature, env.STRIPE_WEBHOOK_SECRET)

        let event;
        try {
            event = JSON.parse(body);
        } catch (err) {
            return new Response('Invalid JSON', { status: 400 });
        }

        // 2. Handle 'checkout.session.completed'
        if (event.type === 'checkout.session.completed') {
            const session = event.data.object;
            const sessionId = session.id;

            console.log(`Payment received for session: ${sessionId}`);

            // 3. Notify the IC Subscription Canister
            try {
                await notifyCanister(sessionId, env);
            } catch (err) {
                console.error('Failed to notify canister:', err);
                return new Response('IC Call Failed', { status: 500 });
            }
        }

        return new Response(JSON.stringify({ received: true }), {
            status: 200,
            headers: { 'Content-Type': 'application/json' },
        });
    },
};

async function notifyCanister(sessionId, env) {
    // Setup Agent
    const agent = new HttpAgent({
        host: env.IC_HOST || 'https://icp0.io',
        // In production, the worker would need an identity to sign the call
        // Or the canister function should be public but verifysessionId.
    });

    // Create Actor for Subscription Canister
    const SUBSCRIPTION_CANISTER_ID = env.SUBSCRIPTION_CANISTER_ID;
    const idlFactory = ({ IDL }) => {
        return IDL.Service({
            'confirm_payment': IDL.Func([IDL.Text], [IDL.Variant({ 'Ok': IDL.Null, 'Err': IDL.Text })], []),
        });
    };

    const actor = Actor.createActor(idlFactory, {
        agent,
        canisterId: SUBSCRIPTION_CANISTER_ID,
    });

    // Call the canister
    const result = await actor.confirm_payment(sessionId);
    if ('Err' in result) {
        throw new Error(result.Err);
    }
}
