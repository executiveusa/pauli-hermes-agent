import { VercelRequest, VercelResponse } from "@vercel/node";
import Stripe from "stripe";

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, { apiVersion: "2023-10-16" });
const webhookSecret = process.env.STRIPE_WEBHOOK_SECRET!;

export const config = { api: { bodyParser: false } };

export default async function handler(req: VercelRequest, res: VercelResponse) {
  if (req.method !== "POST") {
    return res.status(405).end();
  }

  // Read raw body for signature verification
  const rawBody = await readRawBody(req);
  const sig = req.headers["stripe-signature"] as string;

  let event: Stripe.Event;
  try {
    event = stripe.webhooks.constructEvent(rawBody, sig, webhookSecret);
  } catch (err: any) {
    console.error("Webhook signature failed:", err.message);
    return res.status(400).json({ error: `Webhook Error: ${err.message}` });
  }

  switch (event.type) {
    case "payment_intent.succeeded": {
      const intent = event.data.object as Stripe.PaymentIntent;
      await handlePaymentSucceeded(intent);
      break;
    }

    case "customer.subscription.created":
    case "customer.subscription.updated": {
      const sub = event.data.object as Stripe.Subscription;
      await handleSubscriptionActive(sub);
      break;
    }

    case "customer.subscription.deleted": {
      const sub = event.data.object as Stripe.Subscription;
      await handleSubscriptionCanceled(sub);
      break;
    }

    case "invoice.payment_failed": {
      const invoice = event.data.object as Stripe.Invoice;
      await handlePaymentFailed(invoice);
      break;
    }

    default:
      // Ignore unhandled events
      break;
  }

  res.status(200).json({ received: true });
}

async function handlePaymentSucceeded(intent: Stripe.PaymentIntent) {
  const { api_url, api_name, github_repo, customer_email } = intent.metadata || {};

  if (!api_url || !api_name) {
    console.log("Payment succeeded but no generation metadata — manual follow-up needed");
    return;
  }

  // Trigger generation automatically
  await triggerGeneration({
    api_url,
    api_name,
    github_repo,
    payment_intent_id: intent.id,
  });

  if (customer_email) {
    await sendConfirmationEmail(customer_email, api_name, "pay-per-use");
  }
}

async function handleSubscriptionActive(sub: Stripe.Subscription) {
  const customerId = sub.customer as string;
  const customer = await stripe.customers.retrieve(customerId) as Stripe.Customer;

  console.log(`Subscription active: ${sub.id} for ${customer.email}`);

  // Grant unlimited access — store in your DB here
  // await db.upsertSubscription({ stripe_sub_id: sub.id, email: customer.email, status: "active" });

  if (customer.email) {
    await sendConfirmationEmail(customer.email, "PPaaS", "subscription");
  }
}

async function handleSubscriptionCanceled(sub: Stripe.Subscription) {
  console.log(`Subscription canceled: ${sub.id}`);
  // Revoke access in DB
  // await db.updateSubscription({ stripe_sub_id: sub.id, status: "canceled" });
}

async function handlePaymentFailed(invoice: Stripe.Invoice) {
  console.log(`Payment failed: invoice ${invoice.id}`);
  // Optionally send dunning email
}

async function triggerGeneration(opts: {
  api_url: string;
  api_name: string;
  github_repo?: string;
  payment_intent_id: string;
}) {
  const baseUrl = process.env.NEXT_PUBLIC_BASE_URL || "https://ppaas.printingpress.dev";

  const resp = await fetch(`${baseUrl}/api/generate`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(opts.github_repo ? { "x-github-repo": opts.github_repo } : {}),
    },
    body: JSON.stringify({
      api_url: opts.api_url,
      api_name: opts.api_name,
      payment_intent_id: opts.payment_intent_id,
    }),
  });

  if (!resp.ok) {
    console.error("Auto-generation failed:", await resp.text());
  }
}

async function sendConfirmationEmail(to: string, productName: string, type: "pay-per-use" | "subscription") {
  const subject =
    type === "pay-per-use"
      ? `Your ${productName} CLI is being generated`
      : `Welcome to PPaaS — unlimited CLI generation activated`;

  const body =
    type === "pay-per-use"
      ? `Your CLI for ${productName} is generating now. You'll receive a GitHub PR link within 5 minutes.`
      : `Your PPaaS subscription is active. Generate unlimited CLIs at https://ppaas.printingpress.dev`;

  // Use your email service here (Resend, SendGrid, etc.)
  console.log(`Email to ${to}: ${subject}\n${body}`);
}

async function readRawBody(req: VercelRequest): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    req.on("data", (chunk: Buffer) => chunks.push(chunk));
    req.on("end", () => resolve(Buffer.concat(chunks)));
    req.on("error", reject);
  });
}
