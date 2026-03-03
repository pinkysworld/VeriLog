const tracks = [
  {
    label: "Shipping",
    title: "Verifiable audit trail core",
    copy:
      "The base edition already covers the hardest practical foundation: append-only signed records, Merkle roots, proofs, and local verification without a service dependency.",
    next: "Next: freeze golden vectors and publish a compact external verifier."
  },
  {
    label: "Near term",
    title: "Checkpointed archival evidence",
    copy:
      "Signed checkpoints turn the log into something portable. They are the bridge to timestamping, long-term archives, and selective disclosure workflows.",
    next: "Next: add external anchoring targets and bundle schemas."
  },
  {
    label: "Differentiator",
    title: "Private audit telemetry",
    copy:
      "Differential privacy matters here because it can attach to the same evidence surface as integrity, giving operators and auditors a shared trust story.",
    next: "Next: ship schema packs and visible privacy accounting."
  },
  {
    label: "Research",
    title: "Zero-knowledge selective disclosure",
    copy:
      "The long-term edge is proving integrity for ranges and policies without revealing raw events. That is how VeriLog can become genuinely hard to substitute.",
    next: "Next: run a parallel Poseidon commitment experiment before circuit work."
  }
];

const trackGrid = document.getElementById("track-grid");

tracks.forEach((track, index) => {
  const card = document.createElement("article");
  card.className = "panel track-card";
  card.style.transform = `translateY(${index * 6}px)`;
  card.innerHTML = `
    <div class="track-meta">
      <span class="pill">${track.label}</span>
    </div>
    <h3>${track.title}</h3>
    <p class="track-copy">${track.copy}</p>
    <p class="track-next"><strong>${track.next}</strong></p>
  `;
  trackGrid.appendChild(card);
});
