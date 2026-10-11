(function () {
  const templates = {
    why: (context) => '<section class="drawer-section"><p class="section-kicker">Current state</p><h3>' + context.title + '</h3><p>' + context.reason + '</p></section><section class="drawer-section"><h3>Governing condition</h3><p>The displayed state is derived from copied SEA Forge interface evidence. It is not an authoritative backend result.</p></section><section class="drawer-section"><h3>Next lawful paths</h3><ul><li>Inspect supporting evidence.</li><li>Review provenance and freshness.</li><li>Take only the action shown as lawful on the current surface.</li></ul></section>',
    evidence: () => '<section class="drawer-section"><p class="section-kicker">Evidence inventory</p><h3>Source-backed design records</h3><button class="evidence-link" type="button"><strong>Screen contract</strong><span>Primary path</span></button><button class="evidence-link" type="button"><strong>State vocabulary</strong><span>UX epic</span></button><button class="evidence-link" type="button"><strong>Freshness contract</strong><span>Frontend contract</span></button></section><section class="drawer-section"><h3>Verification</h3><p>These references establish interface behavior. They do not represent live backend evidence.</p></section>',
    provenance: () => '<section class="drawer-section"><p class="section-kicker">Projection source</p><h3>Copied SEA Forge specifications</h3><p>This applied kit is derived from the canonical design, mockup brief, atomic breakdown, view-flow, primary-path wireframes, and frontend component contract.</p></section><section class="drawer-section"><h3>Freshness</h3><p>The review projection is current relative to copied project evidence. No runtime service is queried by this static kit.</p></section>',
    record: (context) => '<section class="drawer-section"><p class="section-kicker">Machine-readable example</p><h3>Display model</h3><pre class="drawer-record">state_domain: ' + context.title.toLowerCase().replace(/\\s+/g, "_") + '\nstate: inspectable\nsource_kind: copied_specification\nfreshness: current\ndisplay_only: true</pre></section><p class="section-kicker">No backend record is implied</p><p>This is an interface display model, not an authoritative SEA Forge record.</p>'
  };
  window.SeaForgeComponents = {
    renderDrawerTab(tab, context) {
      const safeContext = context || { title: "Workbench state", reason: "This projection remains inspectable." };
      return (templates[tab] || templates.why)(safeContext);
    }
  };
})();
