// ============================================================
// VeriLog — Site Interactivity
// ============================================================

// ---- Research Tracks Data (R01–R30) ----
const researchTracks = {
  integrity: [
    { id: "R02", name: "ZK Proof of Log Integrity", desc: "Succinct proofs that entry sequences and Merkle roots are consistent" },
    { id: "R04", name: "Merkle-Based Audit Trails", desc: "Fixed-height incremental commitments with membership and range proofs" },
    { id: "R06", name: "Verifiable Deletion with Proof of Erasure", desc: "Delete old logs while preserving cryptographic continuity" },
    { id: "R09", name: "Tamper-Evident Log Chaining", desc: "Rolling window commitments and forward-secure key ratchets" },
    { id: "R10", name: "Long-Term Archival with Proof of Immutability", desc: "Periodic checkpoints reduce audit costs for long archives" },
    { id: "R12", name: "Verifiable Anomaly-Triggered Logging", desc: "Alerts cryptographically bound to detection logic" },
    { id: "R15", name: "ZK Range Queries on Logs", desc: "Prove predicates over log ranges without revealing values" },
    { id: "R21", name: "Recursive Proof Aggregation", desc: "Aggregate multiple log proofs into a single succinct proof" },
    { id: "R22", name: "Post-Quantum Signature Migration", desc: "Transition path from Ed25519 to lattice-based signatures" },
    { id: "R27", name: "Verifiable Log Format Migration", desc: "Prove correctness of format upgrades across schema versions" },
  ],
  privacy: [
    { id: "R03", name: "Per-Event Differential Privacy", desc: "Event-level DP preserving causal order while protecting telemetry" },
    { id: "R07", name: "Regulatory Export Formats", desc: "Automated compliance exports with proof bundles (GDPR/HIPAA/SOC2)" },
    { id: "R08", name: "Cross-Device Correlation with Privacy", desc: "PSI-based correlation without revealing raw device data" },
    { id: "R11", name: "Oblivious Logging Modes", desc: "ORAM-inspired access patterns to hide read operations" },
    { id: "R13", name: "Privacy-Preserving Telemetry Aggregation", desc: "Verifiable aggregates without trusting an aggregator" },
    { id: "R23", name: "Federated Privacy Budget Governance", desc: "Cross-organization DP budget negotiation and enforcement" },
    { id: "R24", name: "Synthetic Telemetry Generation", desc: "Privacy-safe synthetic datasets for testing and sharing" },
    { id: "R28", name: "Privacy-Preserving Log Search", desc: "Encrypted search indexes for querying without decryption" },
  ],
  energy: [
    { id: "R01", name: "Learned Adaptive Logging Frequency", desc: "ML-driven policy reduces energy while maintaining evidence quality" },
    { id: "R05", name: "Energy-Aware Log Compression", desc: "Adaptive compression balancing storage writes and battery" },
    { id: "R17", name: "Energy-Proportional Encryption Strength", desc: "Dynamic cipher selection based on battery and policy" },
    { id: "R18", name: "Learned False-Positive Reduction", desc: "On-device meta-learning to reduce alert fatigue" },
    { id: "R19", name: "Cross-Platform Binary Optimization", desc: "Compile-time and runtime tuning for target architectures" },
    { id: "R25", name: "Hardware-Accelerated Hashing", desc: "Leverage ARM Crypto Extensions and RISC-V for proof speed" },
    { id: "R29", name: "Predictive Maintenance Evidence", desc: "Energy-aware anomaly detection with cryptographic attestation" },
  ],
  federation: [
    { id: "R14", name: "Secure Log Forwarding over Unreliable Links", desc: "Ratcheting encryption with ordering guarantees" },
    { id: "R16", name: "Wasm-Based User-Defined Logging Rules", desc: "Sandboxed custom rules with deterministic execution" },
    { id: "R20", name: "Verifiable Log Sync in Mesh Networks", desc: "Secure gossip keeping partitions consistent with proofs" },
    { id: "R26", name: "Decentralized Timestamping Anchors", desc: "Multiple anchoring targets: transparency logs, blockchains, RFC 3161" },
    { id: "R30", name: "Fleet-Wide Integrity Dashboards", desc: "Aggregated compliance and integrity status across device fleets" },
  ]
};

// ---- Render Research Tracks ----
function renderTracks() {
  Object.entries(researchTracks).forEach(([pillar, tracks]) => {
    const listId = {
      integrity: "tracks-integrity",
      privacy: "tracks-privacy",
      energy: "tracks-energy",
      federation: "tracks-federation"
    }[pillar];

    const container = document.getElementById(listId);
    if (!container) return;

    tracks.forEach(track => {
      const item = document.createElement("div");
      item.className = "track-item";
      item.innerHTML = `
        <span class="track-id">${track.id}</span>
        <div class="track-info">
          <div class="track-name">${track.name}</div>
          <div class="track-desc">${track.desc}</div>
        </div>
      `;
      container.appendChild(item);
    });
  });
}

// ---- Navbar Scroll Effect ----
function initNavbar() {
  const navbar = document.getElementById("navbar");
  if (!navbar) return;

  const update = () => {
    if (window.scrollY > 10) {
      navbar.classList.add("scrolled");
    } else {
      navbar.classList.remove("scrolled");
    }
  };
  window.addEventListener("scroll", update, { passive: true });
  update();
}

// ---- Mobile Navigation Toggle ----
function initMobileNav() {
  const toggle = document.getElementById("navToggle");
  const menu = document.getElementById("navMenu");
  if (!toggle || !menu) return;

  toggle.addEventListener("click", () => {
    menu.classList.toggle("open");
  });

  // Close on link click
  menu.querySelectorAll("a").forEach(link => {
    link.addEventListener("click", () => {
      menu.classList.remove("open");
    });
  });
}

// ---- Smooth Scroll for Anchor Links ----
function initSmoothScroll() {
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener("click", function(e) {
      const target = document.querySelector(this.getAttribute("href"));
      if (target) {
        e.preventDefault();
        target.scrollIntoView({ behavior: "smooth" });
      }
    });
  });
}

// ---- Intersection Observer for Fade-In ----
function initScrollAnimations() {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          entry.target.classList.add("visible");
        }
      });
    },
    { threshold: 0.1, rootMargin: "0px 0px -40px 0px" }
  );

  document.querySelectorAll(
    ".feature-card, .wedge-card, .merkle-card, .timeline-item, .trust-layer, .pillar"
  ).forEach(el => {
    el.style.opacity = "0";
    el.style.transform = "translateY(16px)";
    el.style.transition = "opacity 0.5s ease, transform 0.5s ease";
    observer.observe(el);
  });
}

// Add the CSS class for visible elements
const style = document.createElement("style");
style.textContent = `
  .visible {
    opacity: 1 !important;
    transform: translateY(0) !important;
  }
`;
document.head.appendChild(style);

// ---- Init ----
document.addEventListener("DOMContentLoaded", () => {
  renderTracks();
  initNavbar();
  initMobileNav();
  initSmoothScroll();
  initScrollAnimations();
});
