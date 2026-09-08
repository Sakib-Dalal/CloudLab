<script lang="ts">
  import {
    ArrowLeft,
    ArrowRight,
    ArrowUpRight,
    Clock,
    BookOpen,
    Info,
    Link as LinkIcon,
  } from "@lucide/svelte";
  import SEO from "$lib/components/SEO.svelte";
  import CodeBlock from "$lib/components/CodeBlock.svelte";
  import CommandTabs from "$lib/components/CommandTabs.svelte";
  import { repository } from "$lib/site";
  import type { PageData } from "./$types";
  let { data }: { data: PageData } = $props();
  const guide = $derived(data.guide);
</script>

<SEO title={guide.title} description={guide.description} />
<div class="article-layout">
  <article class="guide-article">
    <div class="breadcrumbs">
      <a href="/docs/">Documentation</a><span>/</span><span
        >{guide.category}</span
      >
    </div>
    <header class="article-header">
      <p class="eyebrow">{guide.category}</p>
      <h1>{guide.title}</h1>
      <p>{guide.description}</p>
      <div class="article-meta">
        <span><Clock size={14} />{guide.minutes} min read</span><span
          ><BookOpen size={14} />CloudLab 2.0</span
        >
      </div>
    </header>
    <details class="mobile-toc">
      <summary>On this page</summary>
      <nav aria-label="On this page">
        {#each guide.sections as section}<a href={`#${section.id}`}
            >{section.title}</a
          >{/each}
      </nav>
    </details>
    {#each guide.sections as section}
      <section id={section.id} class="article-section">
        <h2>
          <a href={`#${section.id}`}
            >{section.title}<LinkIcon
              size={16}
              aria-label="Link to this section"
            /></a
          >
        </h2>
        {#each section.blocks as block}
          {#if block.type === "text"}<p>{block.text}</p>
          {:else if block.type === "list"}
            {#if block.ordered}<ol>
                {#each block.items as item}<li>{item}</li>{/each}
              </ol>{:else}<ul>
                {#each block.items as item}<li>{item}</li>{/each}
              </ul>{/if}
          {:else if block.type === "note"}<aside class="callout">
              <Info size={18} />
              <div>
                <strong>{block.title}</strong>
                <p>{block.text}</p>
              </div>
            </aside>
          {:else if block.type === "code"}<CodeBlock
              code={block.code}
              label={block.label}
            />
          {:else if block.type === "tabs"}<CommandTabs
              options={block.options}
            />
          {:else if block.type === "links"}<div class="article-links">
              {#each block.items as link}<a href={link.href}
                  >{link.label}<ArrowUpRight size={15} /></a
                >{/each}
            </div>
          {:else if block.type === "table"}
            <!-- svelte-ignore a11y_no_noninteractive_tabindex (Scrollable tables must be keyboard accessible on narrow screens.) -->
            <div
              class="table-scroll"
              role="region"
              aria-label={`${section.title} comparison`}
              tabindex="0"
            >
              <table>
                <thead
                  ><tr
                    >{#each block.columns as column}<th scope="col">{column}</th
                      >{/each}</tr
                  ></thead
                ><tbody
                  >{#each block.rows as row}<tr
                      >{#each row as cell, i}{#if i === 0}<th scope="row"
                            >{cell}</th
                          >{:else}<td>{cell}</td>{/if}{/each}</tr
                    >{/each}</tbody
                >
              </table>
            </div>
          {/if}
        {/each}
      </section>
    {/each}
    <div class="article-feedback">
      <span>Found a missing step or something unclear?</span><a
        href={`${repository}/issues`}
        >Help improve this guide<ArrowUpRight size={14} /></a
      >
    </div>
    <nav class="article-pagination" aria-label="Next and previous guides">
      {#if data.previous}<a href={`/docs/${data.previous.slug}/`}
          ><small><ArrowLeft size={13} />Previous guide</small><strong
            >{data.previous.title}</strong
          ></a
        >{:else}<a href="/docs/"
          ><small><ArrowLeft size={13} />Back to</small><strong
            >All documentation</strong
          ></a
        >{/if}{#if data.next}<a href={`/docs/${data.next.slug}/`}
          ><small>Next guide<ArrowRight size={13} /></small><strong
            >{data.next.title}</strong
          ></a
        >{/if}
    </nav>
  </article>
  <aside class="article-toc">
    <nav aria-label="On this page">
      <h2>On this page</h2>
      {#each guide.sections as section}<a href={`#${section.id}`}
          >{section.title}</a
        >{/each}
    </nav>
    <a class="toc-help" href="/docs/troubleshooting/"
      >Need a hand?<span>Open troubleshooting<ArrowUpRight size={13} /></span
      ></a
    >
  </aside>
</div>
