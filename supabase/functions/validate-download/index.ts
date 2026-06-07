import { createClient } from "jsr:@supabase/supabase-js@2";

const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, x-client-info, apikey, content-type",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};

Deno.serve(async (req: Request) => {
  if (req.method === "OPTIONS") {
    return new Response("ok", { headers: corsHeaders });
  }

  try {
    const { license_key, binary_name, platform } = await req.json();

    if (!license_key || !binary_name || !platform) {
      return new Response(
        JSON.stringify({ error: "Missing required fields." }),
        { status: 400, headers: { ...corsHeaders, "Content-Type": "application/json" } }
      );
    }

    const supabase = createClient(
      Deno.env.get("SUPABASE_URL")!,
      Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!
    );

    const { data: license, error: licenseError } = await supabase
      .from("licenses")
      .select("*")
      .eq("license_key", license_key)
      .eq("revoked", false)
      .single();

    if (licenseError || !license) {
      return new Response(
        JSON.stringify({ error: "Invalid or revoked licence key." }),
        { status: 403, headers: { ...corsHeaders, "Content-Type": "application/json" } }
      );
    }

    if (new Date(license.expires_at) < new Date()) {
      return new Response(
        JSON.stringify({ error: "Licence has expired." }),
        { status: 403, headers: { ...corsHeaders, "Content-Type": "application/json" } }
      );
    }

    const { data: binary, error: binaryError } = await supabase
      .from("binaries")
      .select("*")
      .eq("name", binary_name)
      .eq("platform", platform)
      .eq("is_latest", true)
      .single();

    if (binaryError || !binary) {
      return new Response(
        JSON.stringify({ error: "Binary not found for requested platform." }),
        { status: 404, headers: { ...corsHeaders, "Content-Type": "application/json" } }
      );
    }

    if (binary.binary_hash !== license.binary_hash) {
      return new Response(
        JSON.stringify({ error: "Licence is not valid for this binary version." }),
        { status: 403, headers: { ...corsHeaders, "Content-Type": "application/json" } }
      );
    }

    const { data: signedUrl, error: signError } = await supabase
      .storage
      .from("binaries")
      .createSignedUrl(binary.storage_path, 60);

    if (signError || !signedUrl) {
      return new Response(
        JSON.stringify({ error: "Failed to generate download URL." }),
        { status: 500, headers: { ...corsHeaders, "Content-Type": "application/json" } }
      );
    }

    await supabase.from("downloads").insert({
      license_id: license.id,
      binary_name,
      platform,
      ip_hash: req.headers.get("x-forwarded-for") || "unknown",
      user_agent: req.headers.get("user-agent") || "unknown",
    });

    return new Response(
      JSON.stringify({
        download_url: signedUrl.signedUrl,
        sha256: binary.sha256,
        version: binary.version,
        binary_name: binary.name,
        platform: binary.platform,
      }),
      { headers: { ...corsHeaders, "Content-Type": "application/json" } }
    );

  } catch (err) {
    return new Response(
      JSON.stringify({ error: "Internal server error." }),
      { status: 500, headers: { ...corsHeaders, "Content-Type": "application/json" } }
    );
  }
});