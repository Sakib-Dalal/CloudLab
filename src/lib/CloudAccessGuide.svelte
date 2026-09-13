<script lang="ts">
  import {
    ArrowRight,
    Check,
    Cloud,
    Copy,
    Globe2,
    LockKeyhole,
    Server,
    Terminal,
    Box,
    ShieldCheck,
  } from "lucide-svelte";
  import DuckDNSGuide from "./DuckDNSGuide.svelte";
  import { cloudCommand, cloudProviders, publicIp } from "./cloudAccess";
  import type { Snapshot } from "./types";

  let { remote }: { remote?: Snapshot["remote_access"] } = $props();
  let method = $state("cloud");
  let provider = $state("aws");
  let automatic = $state(true);
  let address = $state("");
  let copied = $state("");
  let copyError = $state("");
  const selected = $derived(cloudProviders.find((p) => p.id === provider)!);
  const auto = $derived(automatic && provider !== "other");
  const ip = $derived(publicIp(address));
  const command = $derived(cloudCommand(provider, address, auto));
  const preview = $derived(
    !auto && ip
      ? `https://lab.${ip.replaceAll(".", "-")}.sslip.io`
      : "HTTPS address generated on your VM",
  );

  async function copy(value: string) {
    try {
      await navigator.clipboard.writeText(value);
      copied = value;
      copyError = "";
    } catch {
      copyError = "Select the command and copy it manually in this browser.";
    }
  }
</script>

<div class="cloud-guide">
  <div class="cloud-heading">
    <div class="cloud-symbol"><Cloud size={27} /></div>
    <span>REMOTE ACCESS</span>
  </div>
  <h2>Your cloud. Your lab. Anywhere.</h2>
  <p class="intro">
    Host your dashboard and workspaces together on a cloud VM. Start with its
    public IP and let CloudLab set up the connection.
  </p>
  {#if remote?.public_url}
    <div class="configured">
      <ShieldCheck size={19} />
      <div>
        <strong
          >{remote.managed
            ? "Managed cloud address"
            : "Saved coordinator address"}</strong
        ><span>{remote.public_url}</span>
      </div>
      <span class="status">Configured</span>
    </div>
  {/if}
  <div class="methods" aria-label="Remote access method">
    <button
      class:chosen={method === "cloud"}
      aria-pressed={method === "cloud"}
      onclick={() => (method = "cloud")}><Cloud size={17} />Cloud VM</button
    >
    <button
      class:chosen={method === "duckdns"}
      aria-pressed={method === "duckdns"}
      onclick={() => (method = "duckdns")}
      ><Globe2 size={17} />Home / DuckDNS</button
    >
  </div>

  {#if method === "cloud"}
    <section class="setup-section">
      <div class="step-title">
        <span>01</span>
        <h3>Choose your server</h3>
        <small>Dashboard + compute</small>
      </div>
      <div class="fields">
        <label
          >Cloud provider<select bind:value={provider}
            >{#each cloudProviders as p}<option value={p.id}>{p.name}</option
              >{/each}</select
          ></label
        >
        <div class="address-field">
          {#if provider !== "other"}<label
              >IP address<select bind:value={automatic}
                ><option value={true}>Detect automatically on the VM</option
                ><option value={false}>Use a public / static IP</option></select
              ></label
            >{:else}<span class="field-label">IP address</span>{/if}
          {#if !auto}<label class="ip-label"
              >Public IPv4<input
                aria-invalid={!!address && !ip}
                bind:value={address}
                placeholder="Enter the VM’s public IPv4"
                autocomplete="off"
                spellcheck="false"
              /></label
            >{/if}
        </div>
      </div>
      {#if !auto && address && !ip}<p class="validation" role="alert">
          Enter a routable public IPv4 address without a port. Private and
          example addresses cannot be used.
        </p>{/if}
      <p class="note">
        Reserve a stable address ({selected.address}) to keep your URL after
        restarts. {auto
          ? "Detection runs on your cloud VM when you start setup."
          : "The address must belong to this VM and accept incoming connections."}
      </p>
    </section>

    <div
      class="connection-map"
      role="img"
      aria-label="Your browser opens the public IP, redirects to the HTTPS gateway, and connects to the dashboard and isolated workspaces on the cloud VM."
    >
      <div class="map-node">
        <Globe2 size={22} /><strong>Your browser</strong><small
          >{!auto && ip ? ip : "Public IP"}</small
        >
      </div>
      <ArrowRight size={20} class="map-arrow" />
      <div class="map-node secure">
        <LockKeyhole size={23} /><strong>HTTPS gateway</strong><small
          >Automatic certificates</small
        >
      </div>
      <ArrowRight size={20} class="map-arrow" />
      <div class="map-destinations">
        <span><Server size={17} />Dashboard</span><span
          ><Box size={17} />Workspaces</span
        >
      </div>
      <div class="map-caption">
        <span class="live-dot"></span><span>{preview}</span><small
          >Address preview</small
        >
      </div>
    </div>

    <section class="setup-section">
      <div class="step-title">
        <span>02</span>
        <h3>Let your lab through</h3>
      </div>
      <p class="note">
        Allow these inbound rules in your {selected.firewall} and the VM firewall.
      </p>
      <div class="ports">
        <div>
          <strong>80 <small>TCP</small></strong><span
            >Redirect & certificate setup</span
          >
        </div>
        <div>
          <strong>443 <small>TCP</small></strong><span
            >Dashboard, workspaces & nodes</span
          >
        </div>
        <div class="private-port">
          <ShieldCheck size={19} /><span
            >Keep SSH restricted to your IP.<br />Internal lab ports stay
            private.</span
          >
        </div>
      </div>
    </section>

    <section class="setup-section">
      <div class="step-title">
        <span>03</span>
        <h3>Start CloudLab on the VM</h3>
      </div>
      <p class="note">
        Use Ubuntu 24.04+ or Debian 12+ with Docker Engine, Docker Compose, and
        Python 3 installed. Run this from the CloudLab project folder on that
        VM.
      </p>
      <div class="setup-command">
        <div>
          <span><Terminal size={15} />Run on your cloud VM</span><button
            disabled={!command}
            onclick={() => command && copy(command)}
            aria-label="Copy cloud setup command"
            >{#if command && copied === command}<Check
                size={15}
              />Copied{:else}<Copy size={15} />Copy{/if}</button
          >
        </div>
        <!-- svelte-ignore a11y_no_noninteractive_tabindex (Allows keyboard scrolling of long commands.) -->
        <pre tabindex="0" role="region" aria-label="Cloud setup command"><code
            >{command ||
              "Enter your public IPv4 above to generate the setup command."}</code
          ></pre>
      </div>
      {#if copyError}<p class="validation" role="status">{copyError}</p>{/if}
      <div class="setup-outcomes">
        <span><Check size={15} />Builds all workspace apps</span><span
          ><Check size={15} />Connects this VM as a node</span
        ><span><Check size={15} />Starts again after reboot</span>
      </div>
      <p class="note">
        Setup prints your secure address and a command to read your private
        owner key. Open <strong
          >{!auto && ip ? `http://${ip}` : "http://YOUR_PUBLIC_IP"}</strong
        > to reach the dashboard, then sign in and create a workspace.
      </p>
    </section>
    <details class="cloud-details">
      <summary>How the address works & advanced setup</summary>
      <p>
        Public-IP access redirects to an HTTPS hostname from sslip.io, a public
        DNS service. No domain purchase or DNS token is needed. Every browser
        workspace gets a separate address and certificate. First access can take
        longer while its certificate is issued; certificate service limits still
        apply.
      </p>
      <p>
        To use your own domain, point its <code>lab</code> and wildcard DNS
        records to the VM, then add <code>--domain cloud.example.com</code> to
        setup. For a plan without installing, omit <code>--apply</code>. Full
        prerequisites, updates and troubleshooting are in
        <code>docs/cloud-access.md</code> in your project.
      </p>
      <p>
        This sets up a dedicated VM. It does not move the lab you are currently
        viewing or change your cloud firewall. GPU charts and workspace GPU
        access appear when the VM’s drivers and container runtime support them.
      </p>
    </details>
  {:else}
    <p class="note">
      Connect an existing home lab using a free DuckDNS address, automatic IP
      updates, and an HTTPS gateway. A reachable public IP or VPN is required.
    </p>
    <DuckDNSGuide />
  {/if}
</div>

<style>
  .cloud-guide {
    color: #342e2c;
  }
  .cloud-heading {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 17px;
  }
  .cloud-heading > span {
    font-size: 10px;
    letter-spacing: 2px;
    color: #938477;
    font-weight: 700;
  }
  .cloud-symbol {
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    background: #e5efeb;
    color: #4f8074;
    border-radius: 15px;
  }
  .cloud-guide h2 {
    font-size: 26px;
    letter-spacing: -0.7px;
    margin: 0 0 10px;
  }
  .intro {
    color: #81756d;
    line-height: 1.65;
    font-size: 14px;
    margin: 0 0 22px;
    max-width: 580px;
  }
  .configured {
    display: flex;
    align-items: center;
    gap: 10px;
    border: 1px solid #d4e3db;
    background: #f1f7f3;
    padding: 12px;
    border-radius: 10px;
    margin: 0 0 18px;
    color: #527c6c;
  }
  .configured div {
    min-width: 0;
    flex: 1;
  }
  .configured strong,
  .configured div span {
    display: block;
    font-size: 11px;
    overflow-wrap: anywhere;
  }
  .configured div span {
    margin-top: 5px;
    font-weight: 400;
  }
  .status {
    font-size: 10px;
  }
  .methods {
    display: flex;
    gap: 5px;
    padding: 5px;
    background: #f2ece5;
    border-radius: 10px;
    margin-bottom: 25px;
  }
  .methods button {
    flex: 1;
    padding: 10px;
    border-radius: 7px;
    color: #887b6e;
    font-size: 12px;
  }
  .methods .chosen {
    background: var(--paper);
    color: #413730;
    box-shadow: 0 1px 4px #5b402314;
  }
  .step-title {
    display: flex;
    align-items: center;
    gap: 11px;
    margin: 0 0 15px;
  }
  .step-title > span {
    font-family: monospace;
    font-size: 11px;
    color: #a08365;
    background: #f3e9dc;
    padding: 6px;
    border-radius: 6px;
  }
  h3 {
    font-size: 14px;
    font-weight: 650;
    margin: 0;
  }
  .step-title small {
    margin-left: auto;
    font-size: 10px;
    color: #a19385;
  }
  .fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .fields label,
  .field-label {
    display: block;
    font-size: 11px;
    font-weight: 500;
    margin: 0;
  }
  .fields input,
  .fields select {
    margin-top: 8px;
    font-size: 12px;
    height: 42px;
  }
  .fields .ip-label {
    margin-top: 10px;
  }
  .note {
    font-size: 12px;
    color: #887d73;
    line-height: 1.7;
    margin: 13px 0;
  }
  .note strong {
    font-weight: 550;
    color: #54483d;
    overflow-wrap: anywhere;
  }
  .validation {
    font-size: 12px;
    color: #b14444;
  }
  .connection-map {
    display: grid;
    grid-template-columns: 1fr 24px 1fr 24px 1fr;
    align-items: center;
    gap: 14px;
    background: #252c2d;
    color: #ecede5;
    border-radius: 14px;
    padding: 25px 22px 0;
    margin: 23px 0 26px;
  }
  .map-node {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    text-align: center;
  }
  .map-node strong {
    font-size: 12px;
    font-weight: 500;
  }
  .map-node small {
    font-size: 9px;
    color: #a7b3ae;
  }
  .secure {
    color: #9dd5bf;
  }
  .map-destinations {
    display: grid;
    gap: 9px;
  }
  .map-destinations span {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    padding: 10px 6px;
    border: 1px solid #4b5553;
    border-radius: 7px;
    font-size: 11px;
  }
  .map-caption {
    display: flex;
    align-items: center;
    gap: 8px;
    grid-column: 1/-1;
    font: 10px monospace;
    padding: 16px 0;
    margin-top: 5px;
    border-top: 1px solid #434c4a;
    color: #b8cfc3;
    min-width: 0;
  }
  .map-caption > span:nth-child(2) {
    overflow-wrap: anywhere;
  }
  .map-caption small {
    font:
      9px "DM Sans",
      sans-serif;
    color: #899c93;
    margin-left: auto;
    white-space: nowrap;
  }
  .live-dot {
    width: 5px;
    height: 5px;
    background: #a1b5ab;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .setup-section + .setup-section {
    margin-top: 25px;
  }
  .ports {
    display: grid;
    grid-template-columns: 1fr 1fr 1.3fr;
    gap: 10px;
    margin: 12px 0 25px;
  }
  .ports > div {
    padding: 14px;
    border: 1px solid var(--line);
    border-radius: 9px;
    display: flex;
    flex-direction: column;
    gap: 9px;
  }
  .ports strong {
    font: 20px monospace;
  }
  .ports small {
    font:
      9px "DM Sans",
      sans-serif;
    color: #a3917f;
  }
  .ports span {
    font-size: 10px;
    line-height: 1.6;
    color: #928373;
  }
  .ports .private-port {
    background: #f5f0e9;
    border-color: transparent;
    color: #928373;
  }
  .setup-command {
    border: 1px solid #e4dace;
    border-radius: 10px;
    overflow: hidden;
    background: #f8f2e9;
  }
  .setup-command > div {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 13px;
    background: #efe6d9;
  }
  .setup-command span {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 10px;
    color: #8d7b65;
  }
  .setup-command button {
    font-size: 10px;
    color: #79664c;
  }
  pre {
    font-size: 11px;
    line-height: 1.7;
    padding: 15px;
    margin: 0;
    overflow: auto;
  }
  .setup-outcomes {
    display: flex;
    flex-wrap: wrap;
    gap: 13px;
    margin-top: 14px;
    color: #698c78;
  }
  .setup-outcomes span {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
  }
  .cloud-details {
    border-top: 1px solid var(--line);
    margin-top: 23px;
    padding-top: 17px;
    color: #8a7b6c;
    font-size: 11px;
    line-height: 1.8;
  }
  .cloud-details summary {
    cursor: pointer;
    font-size: 12px;
    color: #6f5f50;
  }
  .cloud-details code {
    font-size: 10px;
  }
  @media (max-width: 600px) {
    .fields {
      grid-template-columns: 1fr;
    }
    .connection-map {
      gap: 6px;
      padding: 20px 10px 0;
      grid-template-columns: 1fr 15px 1fr 15px 1fr;
    }
    .map-destinations span {
      font-size: 9px;
      gap: 4px;
    }
    .map-node strong {
      font-size: 10px;
    }
    .map-node small {
      font-size: 8px;
    }
    .map-caption small {
      display: none;
    }
    .ports {
      grid-template-columns: 1fr 1fr;
    }
    .ports .private-port {
      grid-column: 1/-1;
      flex-direction: row;
      align-items: center;
    }
    .step-title small {
      display: none;
    }
  }
</style>
