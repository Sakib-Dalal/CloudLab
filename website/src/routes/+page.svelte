<script lang="ts">
  import {
    ArrowRight,
    ArrowUpRight,
    BookOpen,
    Code2,
    Terminal,
    Cpu,
    ShieldCheck,
    Network,
    LockKeyhole,
    HardDrive,
    Monitor,
    GitFork,
    ChevronDown,
    Check,
    Box,
  } from "@lucide/svelte";
  import SEO from "$lib/components/SEO.svelte";
  import LabPreview from "$lib/components/LabPreview.svelte";
  import { repository } from "$lib/site";
  const tools = [
    {
      icon: BookOpen,
      number: "01",
      name: "Follow a question.",
      tool: "JupyterLab",
      text: "Turn a hunch into a notebook. Explore data, run Python, and keep your experiments together.",
      className: "notebook",
      slug: "/docs/workspaces/#choose-a-tool",
    },
    {
      icon: Code2,
      number: "02",
      name: "Build what’s next.",
      tool: "code-server",
      text: "Your project, an editor, and a terminal. Pick up your work from a browser on another device.",
      className: "editor",
      slug: "/docs/workspaces/#choose-a-tool",
    },
    {
      icon: Terminal,
      number: "03",
      name: "Make it run.",
      tool: "Linux console",
      text: "A small, contained place to run commands, try scripts, and see where an idea takes you.",
      className: "console",
      slug: "/docs/workspaces/#choose-a-tool",
    },
  ];
  const faqs = [
    [
      "Is CloudLab free?",
      "Yes. CloudLab is open source under the MIT license. You run it on your own hardware. Electricity, a domain, or an optional public gateway can still have costs.",
    ],
    [
      "Do I need a powerful server?",
      "You can start with one computer that runs Docker in Linux-container mode. Available CPU and memory determine which workspaces it can run. Add more devices as your lab grows; a workspace runs on one chosen node.",
    ],
    [
      "Can I reach my lab away from home?",
      "Yes, after you configure a reachable HTTPS gateway and workspace subdomains, or an HTTPS gateway over a VPN. CloudLab does not automatically set up NAT traversal, DNS, or a public tunnel.",
    ],
    [
      "Can collaborators access my personal files?",
      "The API exposes container workspaces with dedicated volumes, not your host folders or desktop. Operators share the workspaces in their assigned lab. Docker shares the host kernel, so use a dedicated VM or machine for untrusted workloads.",
    ],
    [
      "Does anything run on Vercel?",
      "Only this public website and its documentation. Your coordinator, node agents, Docker workspaces, credentials, and project data run on hardware you control.",
    ],
    [
      "Is this a remote desktop tool?",
      "CloudLab borrows the approachable device model of tools like RustDesk and Raspberry Pi Connect, but gives you container-based development tools. It does not share a host desktop or provide host SSH.",
    ],
  ];
</script>

<SEO
  title="Your hardware. One connected lab."
  description="Turn your computers into a personal compute lab. Run isolated JupyterLab, code-server, and Linux workspaces with Rust, Svelte, and Tauri. Open source and self-hosted."
/>

<main id="main">
  <section class="hero container">
    <div class="hero-copy">
      <a class="release-pill" href="/docs/project/"
        ><span class="pill-dot"></span>CloudLab 2.0<span class="pill-divider"
        ></span>A new kind of home lab<ArrowUpRight size={13} /></a
      >
      <p class="eyebrow">YOUR HARDWARE. YOUR POSSIBILITIES.</p>
      <h1>Big ideas.<br />Small machines.<br /><span>One lab.</span></h1>
      <p class="hero-description">
        Bring your computers together. Create isolated workspaces. Build,
        experiment, and explore—from wherever you are.
      </p>
      <div class="hero-actions">
        <a class="button primary large" href="/docs/quickstart/"
          >Build your first lab<ArrowUpRight size={19} /></a
        ><a class="button text-button" href="/docs/introduction/"
          >Take a closer look<ArrowRight size={17} /></a
        >
      </div>
      <div class="hero-reassurance">
        <span><Check size={14} />Open source</span><span
          ><Check size={14} />Self-hosted</span
        ><span><Check size={14} />No CloudLab account</span>
      </div>
    </div>
    <div class="hero-visual">
      <div class="hero-visual-caption">
        <span>THE PERSONAL COMPUTE LAB</span><span>SELF-HOSTED / V2.0</span>
      </div>
      <LabPreview />
      <div class="visual-footnote">
        <span aria-hidden="true">↳</span> Real machines. Container workspaces. A place
        to make things.
      </div>
    </div>
  </section>

  <div class="foundation-strip">
    <div class="container">
      <span>Built with good foundations.</span>
      <div>
        <strong>Rust<span>Backend</span></strong><strong
          >Svelte<span>Interface</span></strong
        ><strong>Tauri<span>Desktop</span></strong><strong
          >Docker<span>Workspaces</span></strong
        >
      </div>
    </div>
  </div>

  <section id="possibilities" class="section container possibilities">
    <div class="section-heading">
      <div>
        <p class="eyebrow">A SPACE FOR EVERY “WHAT IF”</p>
        <h2>Less scattered.<br />More possible.</h2>
      </div>
      <p>
        That spare computer has more to give.<br />Put it to work in a lab that
        feels like yours.
      </p>
    </div>
    <div class="feature-row">
      <article>
        <span class="feature-icon"><Network size={26} strokeWidth={1.6} /></span
        >
        <h3>Many devices. One view.</h3>
        <p>
          See your nodes, their health, and what’s running. Organize different
          projects and trusted groups into separate labs.
        </p>
        <a class="text-link" href="/docs/nodes/"
          >Meet your compute nodes<ArrowUpRight size={16} /></a
        >
      </article>
      <article>
        <span class="feature-icon blue"
          ><Box size={26} strokeWidth={1.6} /></span
        >
        <h3>A workspace, with boundaries.</h3>
        <p>
          Give each experiment a container, a resource budget, and its own
          project storage. Keep work away from your host folders.
        </p>
        <a class="text-link" href="/docs/security/"
          >Understand the isolation<ArrowUpRight size={16} /></a
        >
      </article>
      <article>
        <span class="feature-icon sand"
          ><Monitor size={26} strokeWidth={1.6} /></span
        >
        <h3>Your lab goes where you do.</h3>
        <p>
          Use a browser or the desktop app. Set up your own HTTPS gateway to
          reach your locally hosted lab from another network.
        </p>
        <a class="text-link" href="/docs/remote-access/"
          >Set up remote access<ArrowUpRight size={16} /></a
        >
      </article>
    </div>
  </section>

  <section class="tools-section">
    <div class="container section">
      <div class="section-heading">
        <div>
          <p class="eyebrow">FAMILIAR TOOLS. YOUR OWN SPACE.</p>
          <h2>What will you work on?</h2>
        </div>
        <a class="text-link" href="/docs/workspaces/"
          >Explore workspaces<ArrowUpRight size={17} /></a
        >
      </div>
      <div class="tool-cards">
        {#each tools as tool}<a
            class={`tool-card ${tool.className}`}
            href={tool.slug}
            ><div class="tool-card-top">
              <span>{tool.tool}</span><span>{tool.number} /</span>
            </div>
            <div class="tool-card-icon">
              <tool.icon size={42} strokeWidth={1.3} />
            </div>
            <h3>{tool.name}</h3>
            <p>{tool.text}</p>
            <span class="tool-card-bottom"
              >Inside your container<ArrowUpRight size={20} /></span
            ></a
          >{/each}
      </div>
    </div>
  </section>

  <section class="section container start-section">
    <div class="section-heading">
      <div>
        <p class="eyebrow">FROM SPARE MACHINE TO PERSONAL LAB</p>
        <h2>Start small. Connect more.</h2>
      </div>
      <a class="button secondary" href="/docs/quickstart/"
        >The step-by-step guide<ArrowUpRight size={17} /></a
      >
    </div>
    <ol class="start-steps">
      <li>
        <span class="step-number">01</span>
        <h3>Start your coordinator</h3>
        <p>
          Build CloudLab and open the dashboard. One place to manage your whole
          lab.
        </p>
        <a href="/docs/quickstart/#build-the-coordinator"
          >Install CloudLab<ArrowRight size={15} /></a
        >
      </li>
      <li>
        <span class="step-number">02</span>
        <h3>Bring a computer along</h3>
        <p>
          Prepare Docker, then pair a node with a one-time key. Add others when
          you’re ready.
        </p>
        <a href="/docs/nodes/"
          >Connect your first node<ArrowRight size={15} /></a
        >
      </li>
      <li>
        <span class="step-number">03</span>
        <h3>Make room for an idea</h3>
        <p>
          Pick a tool and a resource budget. Your next workspace is ready to
          take shape.
        </p>
        <a href="/docs/workspaces/"
          >Create a workspace<ArrowRight size={15} /></a
        >
      </li>
    </ol>
  </section>

  <section class="ownership-section container">
    <div class="ownership-copy">
      <p class="eyebrow">LOCAL AT HEART</p>
      <h2>Your compute.<br />Your control.</h2>
      <p>
        No hosted CloudLab control plane. No third-party rendezvous account.
        Your coordinator, project files, and agents live on hardware you choose.
      </p>
      <a class="button cream" href="/docs/architecture/"
        >See how it fits together<ArrowUpRight size={17} /></a
      ><span class="ownership-note"
        >Remote access uses your HTTPS gateway or VPN.</span
      >
    </div>
    <div
      class="ownership-diagram"
      aria-label="Your browser connects to your HTTPS gateway and coordinator, which connects to compute nodes running container workspaces"
    >
      <div class="diagram-client">
        <Monitor size={22} /><span>Your browser or desktop</span>
      </div>
      <div class="diagram-line"><span>HTTPS</span></div>
      <div class="diagram-boundary">
        <span class="boundary-label"
          ><LockKeyhole size={13} />INFRASTRUCTURE YOU CONTROL</span
        >
        <div class="diagram-gateway">
          <Network size={22} />
          <div>
            <strong>Your gateway & coordinator</strong><small
              >Access, orchestration, and app relay</small
            >
          </div>
        </div>
        <div class="diagram-line"><span>OUTBOUND WSS FROM NODES</span></div>
        <div class="diagram-machines">
          <div>
            <Cpu size={21} /><strong>Compute node</strong><span
              ><Box size={12} />Workspace</span
            >
          </div>
          <div>
            <Cpu size={21} /><strong>Compute node</strong><span
              ><Box size={12} />Workspace</span
            >
          </div>
        </div>
      </div>
    </div>
  </section>

  <section class="section container security-section">
    <div>
      <span class="security-emblem"
        ><ShieldCheck size={48} strokeWidth={1.3} /></span
      >
      <p class="eyebrow">THOUGHTFUL BOUNDARIES</p>
      <h2>Share the workspace.<br />Keep the host private.</h2>
      <p>
        Non-root containers, resource limits, dedicated volumes, and separate
        browser origins give each workspace a defined place to run.
      </p>
      <a class="text-link" href="/docs/security/"
        >Read the security model<ArrowUpRight size={17} /></a
      >
    </div>
    <div class="security-details">
      <article>
        <LockKeyhole size={21} />
        <div>
          <h3>Container access by design</h3>
          <p>
            No host shell, home-directory mount, or Docker socket exposed
            through the workspace API.
          </p>
        </div>
      </article>
      <article>
        <HardDrive size={21} />
        <div>
          <h3>Your projects have a home</h3>
          <p>
            Dedicated persistent volumes keep files through stop and resume.
            Manage backups on your nodes.
          </p>
        </div>
      </article>
      <article>
        <ShieldCheck size={21} />
        <div>
          <h3>Trust is part of the setup</h3>
          <p>
            Docker shares the host kernel. CloudLab is for trusted labs; add a
            dedicated VM boundary for untrusted work.
          </p>
        </div>
      </article>
    </div>
  </section>

  <section class="guide-section">
    <div class="section container">
      <div class="section-heading">
        <div>
          <p class="eyebrow">A LITTLE GUIDANCE GOES A LONG WAY</p>
          <h2>You bring the curiosity.<br />We’ll bring the instructions.</h2>
        </div>
        <a class="text-link" href="/docs/"
          >Browse all guides<ArrowUpRight size={17} /></a
        >
      </div>
      <div class="guide-links">
        <a href="/docs/quickstart/"
          ><span class="guide-index">01 / START HERE</span>
          <h3>Your first lab</h3>
          <p>From source code to a running workspace, one step at a time.</p>
          <span>10 min read<ArrowUpRight size={20} /></span></a
        ><a href="/docs/remote-access/"
          ><span class="guide-index">02 / GO FURTHER</span>
          <h3>Reach it from anywhere</h3>
          <p>Set up HTTPS, workspace subdomains, and your own gateway.</p>
          <span>9 min read<ArrowUpRight size={20} /></span></a
        ><a href="/docs/access-and-settings/"
          ><span class="guide-index">03 / MAKE IT YOURS</span>
          <h3>A lab that fits your work</h3>
          <p>Tune resource defaults, access roles, and session policies.</p>
          <span>6 min read<ArrowUpRight size={20} /></span></a
        >
      </div>
    </div>
  </section>

  <section class="section container faq-section">
    <div>
      <p class="eyebrow">GOOD QUESTIONS</p>
      <h2>A few things<br />you might wonder.</h2>
      <p>Something else on your mind?</p>
      <a class="text-link" href={`${repository}/issues`}
        >Ask on GitHub<ArrowUpRight size={16} /></a
      >
    </div>
    <div class="faq-list">
      {#each faqs as [question, answer]}<details>
          <summary>{question}<ChevronDown size={18} /></summary>
          <p>{answer}</p>
        </details>{/each}
    </div>
  </section>

  <section class="final-cta container">
    <span class="eyebrow">THERE’S A LAB IN THERE SOMEWHERE.</span>
    <h2>Give your next idea<br />a place to run.</h2>
    <div>
      <a class="button primary large" href="/docs/quickstart/"
        >Let’s build your lab<ArrowUpRight size={19} /></a
      ><a class="button secondary large" href={repository}
        ><GitFork size={18} />Explore the source</a
      >
    </div>
    <p>Open source. Self-hosted. Ready for your curiosity.</p>
  </section>
</main>
