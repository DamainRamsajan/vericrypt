// deploy: 1780839046
import "@supabase/functions-js/edge-runtime.d.ts";
import { withSupabase } from "@supabase/server";

export default {
  fetch: withSupabase({
    auth: ["secret"]
  }, async (req, ctx) => {
    const { email, organization, contact_name, country, tier = "B", validity_days = 365, max_assets = 100000 } = await req.json();

    if (!email || !organization) {
      return Response.json({ error: "email and organization are required." }, { status: 400 });
    }

    // Upsert customer
    const { data: customer, error: customerError } = await ctx.supabaseAdmin
      .from("customers")
      .upsert({
        email,
        organization,
        contact_name: contact_name || null,
        country: country || null,
        tier: tier === "A" ? "trial" : tier === "D" ? "enterprise" : "standard",
      }, { onConflict: "email" })
      .select()
      .single();

    if (customerError || !customer) {
      return Response.json({ error: "Failed to create customer record." }, { status: 500 });
    }

    // Get latest binary hash
    const { data: latestBinary } = await ctx.supabaseAdmin
      .from("binaries")
      .select("binary_hash")
      .eq("name", "vericrypt")
      .eq("is_latest", true)
      .single();

    const binaryHash = latestBinary?.binary_hash || "latest";

    const now = new Date();
    const expires = new Date(now.getTime() + validity_days * 24 * 60 * 60 * 1000);

    const claims = {
      sub: customer.id,
      org: organization,
      email,
      tier,
      binary_hash: binaryHash,
      max_assets,
      features: tier === "D" ? ["tee_attestation", "verichain_sth", "custom_axioms", "white_label_verifier"] :
                tier === "C" ? ["tee_attestation", "verichain_sth", "custom_axioms"] :
                tier === "B" ? ["tee_attestation", "verichain_sth"] : [],
      iat: now.toISOString(),
      exp: expires.toISOString(),
    };

    // Generate PASETO v4.public token
    const privateKeyHex = Deno.env.get("PASETO_PRIVATE_KEY")!;
    const privateKeyBytes = new Uint8Array(
      privateKeyHex.match(/.{1,2}/g)!.map(b => parseInt(b, 16))
    );

    // Manual PASETO v4.public signing (Ed25519)
    const header = { alg: "v4.public", typ: "JWT" };
    const payload = claims;
    const footer = "";

    const h = btoa(JSON.stringify(header)).replace(/=+$/, "").replace(/\+/g, "-").replace(/\//g, "_");
    const p = btoa(JSON.stringify(payload)).replace(/=+$/, "").replace(/\+/g, "-").replace(/\//g, "_");
    const f = btoa(footer).replace(/=+$/, "").replace(/\+/g, "-").replace(/\//g, "_");

    const m = `${h}.${p}.${f}`;

    // Sign with Ed25519 using Web Crypto
    const key = await crypto.subtle.importKey(
      "raw",
      privateKeyBytes.slice(0, 32),
      { name: "Ed25519" },
      false,
      ["sign"]
    );

    const signature = await crypto.subtle.sign(
      "Ed25519",
      key,
      new TextEncoder().encode(m)
    );

    const sigB64 = btoa(String.fromCharCode(...new Uint8Array(signature)))
      .replace(/=+$/, "").replace(/\+/g, "-").replace(/\//g, "_");

    const token = `v4.public.${h}.${p}.${sigB64}.${f}`;

    // Store license
    const { error: licenseError } = await ctx.supabaseAdmin
      .from("licenses")
      .insert({
        customer_id: customer.id,
        license_key: token,
        binary_hash: binaryHash,
        issued_at: now.toISOString(),
        expires_at: expires.toISOString(),
        max_assets,
        features: claims.features,
      });

    if (licenseError) {
      return Response.json({ error: "Failed to store license record." }, { status: 500 });
    }

    // Send email via Resend
    try {
      await fetch("https://api.resend.com/emails", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${Deno.env.get("RESEND_API_KEY")}`,
        },
        body: JSON.stringify({
          from: "VeriCrypt <licenses@vericrypt.io>",
          to: email,
          subject: `Your VeriCrypt Licence Key — ${organization}`,
          html: `
            <h2>Your VeriCrypt Licence Key</h2>
            <p>Dear ${contact_name || "Customer"},</p>
            <p>Your VeriCrypt licence key is ready. Use this token on the <a href="https://vericrypt.pages.dev/download.html">Download Portal</a> to access your binaries.</p>
            <p><strong>Licence Key:</strong></p>
            <pre>${token}</pre>
            <p><strong>Tier:</strong> ${tier}<br>
            <strong>Organization:</strong> ${organization}<br>
            <strong>Expires:</strong> ${expires.toISOString()}</p>
            <p><strong>Next Steps:</strong></p>
            <ol>
              <li>Go to <a href="https://vericrypt.pages.dev/download.html">vericrypt.pages.dev/download.html</a></li>
              <li>Enter your licence key</li>
              <li>Download the vericrypt and vericrypt-verify binaries</li>
              <li>Follow the <a href="https://vericrypt.pages.dev/docs/implementation.html">Implementation Manual</a></li>
            </ol>
            <p>After downloading, run <code>./vericrypt activate --key YOUR_TOKEN</code> on your air-gapped host.</p>
            <p>Regards,<br>VeriCrypt Channel Sales</p>
          `,
        }),
      });
    } catch (emailErr) {
      console.error("Email send failed:", emailErr);
    }

    return Response.json({
      license_key: token,
      customer_id: customer.id,
      organization,
      tier,
      expires_at: expires.toISOString(),
    });
  })
};