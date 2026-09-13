<script lang="ts">
  import { duckdnsGuide } from "../../shared/duckdns-guide";

  let platform = $state(0);
  let copied = $state("");
  let copyError = $state("");
  async function copy(code: string) {
    try {
      await navigator.clipboard.writeText(code);
      copied = code;
      copyError = "";
    } catch {
      copyError =
        "Select the command text and copy it manually in this browser.";
    }
  }
</script>

{#snippet command(code: string, label: string)}
  <div class="guide-command">
    <div class="command-heading">
      <span>{label}</span><button
        type="button"
        onclick={() => copy(code)}
        aria-label={`Copy ${label} commands`}
        >{copied === code ? "Copied" : "Copy"}</button
      >
    </div>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex (Keyboard users need to scroll long commands.) -->
    <pre tabindex="0" role="region" aria-label={`${label} commands`}><code
        >{code}</code
      ></pre>
  </div>
{/snippet}

<details class="full-guide">
  <summary>Read all 10 DuckDNS setup steps</summary>
  <div class="guide-body">
    <p class="guide-intro">
      The complete guide is included in CloudLab. Expand each step as you work
      through it.
    </p>
    <label class="platform-choice"
      >Commands for
      <select bind:value={platform}>
        <option value={0}>Linux / macOS</option>
        <option value={1}>Windows PowerShell</option>
      </select>
    </label>
    {#if copyError}<p role="status">{copyError}</p>{/if}
    {#each duckdnsGuide.sections as section}
      <details class="guide-step">
        <summary>{section.title}</summary>
        <div class="step-body">
          {#each section.blocks as block}
            {#if block.type === "text"}<p>{block.text}</p>
            {:else if block.type === "list"}
              {#if block.ordered}<ol>
                  {#each block.items as item}<li>{item}</li>{/each}
                </ol>
              {:else}<ul>
                  {#each block.items as item}<li>{item}</li>{/each}
                </ul>{/if}
            {:else if block.type === "note"}<div class="guide-note" role="note">
                <strong>{block.title}</strong>
                <p>{block.text}</p>
              </div>
            {:else if block.type === "code"}{@render command(
                block.code,
                block.label ?? "Terminal",
              )}
            {:else if block.type === "tabs"}
              {@const option = block.options[platform] ?? block.options[0]}
              {@render command(option.code, option.label)}
            {:else if block.type === "table"}<dl>
                {#each block.rows as row}<div>
                    <dt>{row[0]}</dt>
                    <dd>{row.slice(1).join(" · ")}</dd>
                  </div>{/each}
              </dl>
            {:else if block.type === "links"}<div class="guide-links">
                {#each block.items as link}<a
                    href={link.href.startsWith("/")
                      ? `https://cloudlab-alpha.vercel.app${link.href}`
                      : link.href}
                    target="_blank"
                    rel="noopener noreferrer">{link.label} ↗</a
                  >{/each}
              </div>
            {/if}
          {/each}
        </div>
      </details>
    {/each}
  </div>
</details>

<style>
  .full-guide {
    margin: 24px 0 16px;
    border: 1px solid var(--line);
    border-radius: 12px;
    overflow: hidden;
  }
  summary {
    cursor: pointer;
    font-weight: 650;
    padding: 16px;
    line-height: 1.5;
  }
  .full-guide > summary {
    background: var(--cream);
    color: var(--red);
  }
  .guide-body {
    padding: 0 16px 16px;
  }
  .guide-intro {
    margin: 16px 0;
  }
  .platform-choice {
    display: grid;
    gap: 8px;
  }
  .guide-step {
    border-top: 1px solid var(--line);
  }
  .guide-step > summary {
    padding: 16px 0;
  }
  .step-body {
    padding-bottom: 16px;
    min-width: 0;
  }
  p,
  li,
  dd {
    font-size: 13px;
    line-height: 1.8;
    overflow-wrap: anywhere;
  }
  p {
    margin: 12px 0;
  }
  li {
    padding-left: 3px;
    margin: 10px 0;
  }
  ol,
  ul {
    padding-left: 20px;
  }
  .guide-note {
    background: var(--cream);
    border-radius: 8px;
    padding: 14px;
    margin: 16px 0;
  }
  .guide-note strong {
    font-size: 13px;
  }
  .guide-note p {
    margin-bottom: 0;
  }
  dl > div {
    padding: 12px 0;
    border-bottom: 1px solid var(--line);
  }
  dt {
    font-size: 12px;
    font-weight: 650;
  }
  dd {
    margin: 4px 0 0;
  }
  .guide-links {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 16px;
  }
  a {
    color: var(--red);
    font-size: 12px;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .guide-command {
    margin: 16px 0;
    overflow: hidden;
    border-radius: 8px;
    background: #302d29;
    color: #f7f3eb;
  }
  .command-heading {
    padding: 9px 12px;
    border-bottom: 1px solid #ffffff26;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
    font-size: 11px;
  }
  .command-heading button {
    padding: 5px 9px;
    color: inherit;
    background: #ffffff12;
    border: 1px solid #ffffff38;
    border-radius: 5px;
    font-size: 11px;
    flex-shrink: 0;
  }
  pre {
    margin: 0;
    padding: 14px;
    overflow-x: auto;
    font-size: 11px;
    line-height: 1.75;
    white-space: pre;
  }
</style>
