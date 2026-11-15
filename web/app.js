(() => {
  'use strict';

  // Estado global de la aplicación
  const state = {
    tools: [],
    lexiconCount: 0,
    activeView: 'catalogo',
    activeToolId: 'palabrea',
    activeTab: 'cards',
    currentPreview: null,
    selectedCardNumber: null,
    selectedCategoryLabel: '',
    selectedWordIndex: null,
    zoomLevel: 100,
    materials: [],
  };

  // Referencias DOM
  const els = {
    navBtns: document.querySelectorAll('.nav-btn'),
    views: document.querySelectorAll('.view-section'),
    statusChipText: document.getElementById('communicator-status-text'),
    themeToggleBtn: document.getElementById('theme-toggle-btn'),
    libraryCountBadge: document.getElementById('library-count-badge'),

    // Catálogo
    statToolsCount: document.getElementById('stat-tools-count'),
    statLexiconCount: document.getElementById('stat-lexicon-count'),
    statPdfsCount: document.getElementById('stat-pdfs-count'),
    catalogSearchInput: document.getElementById('catalog-search-input'),
    catalogToolsGrid: document.getElementById('catalog-tools-grid'),
    rescanToolsBtn: document.getElementById('rescan-tools-btn'),

    // Estudio Sidebar
    studioToolSelector: document.getElementById('studio-tool-selector'),
    studioToolDesc: document.getElementById('studio-tool-desc'),
    studioParamsForm: document.getElementById('studio-params-form'),
    activeSeedCode: document.getElementById('active-seed-code'),
    randomSeedBtn: document.getElementById('random-seed-btn'),
    generateDownloadBtn: document.getElementById('generate-download-btn'),
    saveLibraryOnlyBtn: document.getElementById('save-library-only-btn'),

    // KPIs
    kpiTotalCards: document.getElementById('kpi-total-cards'),
    kpiTotalPages: document.getElementById('kpi-total-pages'),
    kpiPlacedWords: document.getElementById('kpi-placed-words'),
    kpiSynAntPairs: document.getElementById('kpi-syn-ant-pairs'),
    summaryCategoryPills: document.getElementById('summary-category-pills'),

    // Pestañas Estudio
    previewTabs: document.querySelectorAll('.preview-tab'),
    previewPanes: document.querySelectorAll('.preview-pane'),
    tabCardsCount: document.getElementById('tab-cards-count'),
    tabWordsCount: document.getElementById('tab-words-count'),
    activeSelectionBanner: document.getElementById('active-selection-banner'),
    activeSelectionText: document.getElementById('active-selection-text'),
    clearHighlightBtn: document.getElementById('clear-highlight-btn'),

    // Tarjetas
    toggleCardSolutions: document.getElementById('toggle-card-solutions'),
    cardsPagesContainer: document.getElementById('cards-pages-container'),

    // Nube de palabras SVG e Inspector
    wcCardHighlightSelect: document.getElementById('wc-card-highlight-select'),
    wcCategoryFilterSelect: document.getElementById('wc-category-filter-select'),
    wcZoomOutBtn: document.getElementById('wc-zoom-out-btn'),
    wcZoomInBtn: document.getElementById('wc-zoom-in-btn'),
    wcZoomResetBtn: document.getElementById('wc-zoom-reset-btn'),
    wcZoomLabel: document.getElementById('wc-zoom-label'),
    svgSheetWrapper: document.getElementById('svg-sheet-wrapper'),
    wordcloudSvg: document.getElementById('wordcloud-svg'),
    inspectorEmptyHint: document.getElementById('inspector-empty-hint'),
    inspectorDetails: document.getElementById('inspector-details'),
    inspWord: document.getElementById('insp-word'),
    inspCategory: document.getElementById('insp-category'),
    inspSyllables: document.getElementById('insp-syllables'),
    inspLetters: document.getElementById('insp-letters'),
    inspVowels: document.getElementById('insp-vowels'),
    inspConsonants: document.getElementById('insp-consonants'),
    inspSynonyms: document.getElementById('insp-synonyms'),
    inspAntonyms: document.getElementById('insp-antonyms'),
    inspMatchingCards: document.getElementById('insp-matching-cards'),

    // Solucionario Docente
    teacherSearchInput: document.getElementById('teacher-search-input'),
    printTeacherKeyBtn: document.getElementById('print-teacher-key-btn'),
    teacherSolutionsTbody: document.getElementById('teacher-solutions-tbody'),

    // Visor PDF
    directPdfOpenLink: document.getElementById('direct-pdf-open-link'),
    pdfPreviewIframe: document.getElementById('pdf-preview-iframe'),

    // Biblioteca
    refreshLibraryBtn: document.getElementById('refresh-library-btn'),
    libraryFilesTbody: document.getElementById('library-files-tbody'),
    libraryViewerTitle: document.getElementById('library-viewer-title'),
    libraryViewerDownload: document.getElementById('library-viewer-download'),
    libraryPdfIframe: document.getElementById('library-pdf-iframe'),

    // Comunicador
    commRegisteredList: document.getElementById('comm-registered-list'),

    // Toast
    toast: document.getElementById('toast-notification'),
  };

  let toastTimer = null;
  function showToast(message) {
    if (!els.toast) return;
    els.toast.textContent = message;
    els.toast.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      els.toast.hidden = true;
    }, 3200);
  }

  function escapeHtml(str) {
    return String(str ?? '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function formatBytes(bytes) {
    if (!bytes || bytes < 1024) return `${bytes || 0} B`;
    const kb = bytes / 1024;
    if (kb < 1024) return `${kb.toFixed(1)} KB`;
    return `${(kb / 1024).toFixed(2)} MB`;
  }

  // =========================================================================
  // NAVEGACIÓN ENTRE VISTAS Y PESTAÑAS
  // =========================================================================

  function switchView(viewName) {
    state.activeView = viewName;
    els.navBtns.forEach((btn) => {
      btn.classList.toggle('active', btn.dataset.view === viewName);
    });
    els.views.forEach((sec) => {
      sec.classList.toggle('active', sec.id === `view-${viewName}`);
    });

    if (viewName === 'biblioteca') {
      loadMaterialsLibrary();
    } else if (viewName === 'estudio' && state.activeTab === 'pdf') {
      refreshPdfIframe();
    }
  }

  function switchStudioTab(tabName) {
    state.activeTab = tabName;
    els.previewTabs.forEach((tab) => {
      tab.classList.toggle('active', tab.dataset.tab === tabName);
    });
    els.previewPanes.forEach((pane) => {
      pane.classList.toggle('active', pane.id === `pane-${tabName}`);
    });

    if (tabName === 'pdf') {
      refreshPdfIframe();
    }
  }

  // =========================================================================
  // CARGA DEL COMUNICADOR Y CATÁLOGO DE HERRAMIENTAS
  // =========================================================================

  async function loadCommunicatorCatalog() {
    try {
      const res = await fetch('/api/tools');
      if (!res.ok) throw new Error('Error al consultar /api/tools');
      const data = await res.json();

      state.tools = data.tools || [];
      state.lexiconCount = data.lexicon_count || 0;

      els.statusChipText.textContent = `${state.tools.length} herramientas activas · Motor Rust OK`;
      els.statToolsCount.textContent = String(state.tools.length);
      els.statLexiconCount.textContent = `${state.lexiconCount} palabras`;

      renderCatalogGrid();
      renderCommunicatorView();
      populateStudioToolSelector();

      if (state.tools.length > 0) {
        const exists = state.tools.some((t) => t.id === state.activeToolId);
        if (!exists) {
          state.activeToolId = state.tools[0].id;
        }
        selectToolInStudio(state.activeToolId);
      }
    } catch (err) {
      els.statusChipText.textContent = 'Error de conexión con backend';
      console.error(err);
    }
  }

  function renderCatalogGrid() {
    const query = (els.catalogSearchInput.value || '').trim().toLowerCase();
    const filtered = state.tools.filter((tool) => {
      if (!query) return true;
      const haystack = `${tool.name} ${tool.subtitle} ${tool.description} ${tool.category} ${tool.age_range}`.toLowerCase();
      return haystack.includes(query);
    });

    if (filtered.length === 0) {
      els.catalogToolsGrid.innerHTML = `
        <div class="sidebar-card">
          <h3>No se encontraron herramientas</h3>
          <p class="tool-desc">Prueba con otro término de búsqueda o sincroniza el comunicador.</p>
        </div>
      `;
      return;
    }

    const capLabels = {
      live_preview: '⚡ Vista Previa en Vivo',
      wordcloud_svg: '☁️ Nube de Palabras SVG',
      teacher_key: '🎓 Hoja del Docente',
      pdf_export: '📄 Exportación PDF Vectorial',
    };

    els.catalogToolsGrid.innerHTML = filtered
      .map((tool) => {
        const capsHtml = (tool.capabilities || [])
          .map((c) => `<span class="cap-tag">${escapeHtml(capLabels[c] || c)}</span>`)
          .join('');

        const iconSymbol =
          tool.icon === 'sparkles'
            ? '✨'
            : tool.icon === 'calculator'
            ? '🧮'
            : tool.icon === 'book-open'
            ? '📘'
            : '🛠️';

        return `
          <article class="tool-card">
            <div>
              <div class="tool-card-top">
                <div class="tool-icon-box" aria-hidden="true">${iconSymbol}</div>
                <div class="tool-badges">
                  <span class="badge-chip native">${escapeHtml(tool.badge)}</span>
                  <span class="badge-chip">${escapeHtml(tool.category)}</span>
                </div>
              </div>
              <h3>${escapeHtml(tool.name)}</h3>
              <p class="tool-subtitle">${escapeHtml(tool.subtitle)}</p>
              <p class="tool-desc">${escapeHtml(tool.description)}</p>
              <div class="tool-meta-row">
                <span class="cap-tag">🎯 ${escapeHtml(tool.age_range)}</span>
                ${capsHtml}
              </div>
            </div>
            <div class="tool-card-footer">
              <button type="button" class="primary-btn full-width open-studio-btn" data-tool-id="${escapeHtml(tool.id)}">
                Abrir en Estudio &amp; Vista Previa
              </button>
              <button type="button" class="secondary-btn quick-pdf-btn" data-tool-id="${escapeHtml(tool.id)}" title="Generar y descargar PDF con valores por defecto">
                ⬇ PDF Rápido
              </button>
            </div>
          </article>
        `;
      })
      .join('');

    els.catalogToolsGrid.querySelectorAll('.open-studio-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const toolId = btn.dataset.toolId;
        selectToolInStudio(toolId);
        switchView('estudio');
      });
    });

    els.catalogToolsGrid.querySelectorAll('.quick-pdf-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const toolId = btn.dataset.toolId;
        const url = `/api/tools/${encodeURIComponent(toolId)}/pdf?save=1&download=1`;
        window.open(url, '_blank');
        setTimeout(loadMaterialsLibrary, 600);
        showToast('PDF generado y añadido a la biblioteca.');
      });
    });
  }

  function renderCommunicatorView() {
    els.commRegisteredList.innerHTML = state.tools
      .map(
        (tool) => `
        <div class="comm-item">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.35rem;">
            <strong>${escapeHtml(tool.name)} (<code>${escapeHtml(tool.id)}</code>)</strong>
            <span class="badge-chip native">${escapeHtml(tool.source_kind)} · v${escapeHtml(tool.version)}</span>
          </div>
          <p style="font-size:0.82rem;color:var(--text-secondary);margin-bottom:0.4rem;">
            ${escapeHtml(tool.subtitle)}
          </p>
          <div style="font-size:0.76rem;color:var(--text-muted);">
            Parámetros expuestos: <code>${(tool.params || []).map((p) => p.name).join(', ')}</code>
          </div>
        </div>
      `
      )
      .join('');
  }

  // =========================================================================
  // ESTUDIO DE GENERACIÓN: FORMULARIO DINÁMICO Y VISTA PREVIA
  // =========================================================================

  function populateStudioToolSelector() {
    els.studioToolSelector.innerHTML = state.tools
      .map((t) => `<option value="${escapeHtml(t.id)}">${escapeHtml(t.name)}</option>`)
      .join('');
    els.studioToolSelector.value = state.activeToolId;
  }

  function selectToolInStudio(toolId) {
    const tool = state.tools.find((t) => t.id === toolId);
    if (!tool) return;

    state.activeToolId = tool.id;
    els.studioToolSelector.value = tool.id;
    els.studioToolDesc.textContent = tool.subtitle;

    buildDynamicParamsForm(tool);
    fetchLivePreview(false);
  }

  function buildDynamicParamsForm(tool) {
    els.studioParamsForm.innerHTML = (tool.params || [])
      .map((param) => {
        const inputId = `param-input-${param.name}`;
        let controlHtml = '';

        if (param.type === 'number') {
          controlHtml = `
            <input
              type="number"
              id="${inputId}"
              name="${escapeHtml(param.name)}"
              class="number-control"
              min="${param.min ?? 1}"
              max="${param.max ?? 120}"
              step="${param.step ?? 1}"
              value="${escapeHtml(param.default_value)}"
            />
          `;
        } else if (param.type === 'select') {
          const opts = (param.options || [])
            .map(
              (o) =>
                `<option value="${escapeHtml(o.value)}" ${
                  o.value === param.default_value ? 'selected' : ''
                }>${escapeHtml(o.label)}</option>`
            )
            .join('');
          controlHtml = `
            <select id="${inputId}" name="${escapeHtml(param.name)}" class="select-control">
              ${opts}
            </select>
          `;
        } else {
          controlHtml = `
            <input
              type="text"
              id="${inputId}"
              name="${escapeHtml(param.name)}"
              class="text-control"
              placeholder="${escapeHtml(param.placeholder || '')}"
              value="${escapeHtml(param.default_value)}"
            />
          `;
        }

        return `
          <div class="form-group">
            <label for="${inputId}" class="field-label">${escapeHtml(param.label)}</label>
            ${controlHtml}
            <small>${escapeHtml(param.description)}</small>
          </div>
        `;
      })
      .join('');

    // Actualizar vista previa en vivo cuando cambie cualquier parámetro
    els.studioParamsForm.querySelectorAll('input, select').forEach((input) => {
      input.addEventListener('change', () => {
        if (input.name !== 'filename') {
          fetchLivePreview(false);
        } else if (state.activeTab === 'pdf') {
          refreshPdfIframe();
        }
      });
    });
  }

  function collectQueryParams(forceNewRandomSeed = false) {
    const params = new URLSearchParams();
    const formData = new FormData(els.studioParamsForm);

    for (const [k, v] of formData.entries()) {
      const trimmed = String(v).trim();
      if (k === 'seed' && forceNewRandomSeed) {
        continue;
      }
      if (trimmed !== '') {
        params.set(k, trimmed);
      }
    }

    if (forceNewRandomSeed) {
      const randomSeed = Math.floor(100000 + Math.random() * 8999999);
      params.set('seed', String(randomSeed));
      const seedInput = els.studioParamsForm.querySelector('[name="seed"]');
      if (seedInput) {
        seedInput.value = String(randomSeed);
      }
    } else if (!params.has('seed') && state.currentPreview && state.currentPreview.seed) {
      params.set('seed', String(state.currentPreview.seed));
    }

    return params;
  }

  async function fetchLivePreview(forceNewRandomSeed = false) {
    try {
      const params = collectQueryParams(forceNewRandomSeed);
      const url = `/api/tools/${encodeURIComponent(state.activeToolId)}/preview?${params.toString()}`;
      const res = await fetch(url);
      if (!res.ok) throw new Error('Error al obtener la vista previa');
      const preview = await res.json();

      state.currentPreview = preview;
      state.selectedCardNumber = null;
      state.selectedCategoryLabel = '';
      state.selectedWordIndex = null;

      // Sincronizar semilla activa
      els.activeSeedCode.textContent = String(preview.seed);
      const seedInput = els.studioParamsForm.querySelector('[name="seed"]');
      if (seedInput && !seedInput.value) {
        seedInput.value = String(preview.seed);
      }

      renderStudioSummary(preview);
      renderCardsPreview(preview);
      renderWordCloudControls(preview);
      renderWordCloudSvg(preview);
      renderTeacherSolutionsTable(preview);
      updateSelectionBanner();

      if (state.activeTab === 'pdf') {
        refreshPdfIframe();
      }
    } catch (err) {
      console.error(err);
      showToast('No se pudo actualizar la vista previa.');
    }
  }

  function renderStudioSummary(preview) {
    const sum = preview.summary || {};
    els.kpiTotalCards.textContent = String(sum.total_cards ?? 0);
    els.kpiTotalPages.textContent = String(sum.total_pages ?? 0);
    els.kpiPlacedWords.textContent = String(sum.placed_words_count ?? 0);
    els.kpiSynAntPairs.textContent = String((sum.synonym_pairs_count || 0) + (sum.antonym_pairs_count || 0));

    els.tabCardsCount.textContent = String(sum.total_cards ?? 0);
    els.tabWordsCount.textContent = String(sum.placed_words_count ?? 0);

    const cats = sum.categories || [];
    els.summaryCategoryPills.innerHTML = cats
      .map(
        (c) => `
        <button type="button" class="cat-pill ${
          state.selectedCategoryLabel === c.label ? 'active' : ''
        }" data-cat="${escapeHtml(c.label)}">
          ${escapeHtml(c.label)} (${c.count})
        </button>
      `
      )
      .join('');

    els.summaryCategoryPills.querySelectorAll('.cat-pill').forEach((pill) => {
      pill.addEventListener('click', () => {
        const cat = pill.dataset.cat;
        if (state.selectedCategoryLabel === cat) {
          state.selectedCategoryLabel = '';
        } else {
          state.selectedCategoryLabel = cat;
          state.selectedCardNumber = null;
        }
        els.wcCategoryFilterSelect.value = state.selectedCategoryLabel;
        els.wcCardHighlightSelect.value = '';
        switchStudioTab('wordcloud');
        applyWordCloudHighlights();
        renderStudioSummary(state.currentPreview);
      });
    });
  }

  // =========================================================================
  // PESTAÑA 1: RENDERIZADO DE TARJETAS DIDÁCTICAS EN CUADRÍCULA 3x2
  // =========================================================================

  function renderCardsPreview(preview) {
    const cards = preview.cards || [];
    const showSolutions = els.toggleCardSolutions.checked;
    const hasWordCloud = (preview.placed_words || []).length > 0;

    const chunks = [];
    for (let i = 0; i < cards.length; i += 6) {
      chunks.push(cards.slice(i, i + 6));
    }

    els.cardsPagesContainer.innerHTML = chunks
      .map((chunk, pageIdx) => {
        const cardsHtml = chunk
          .map((card) => {
            const isSelected = state.selectedCardNumber === card.number;
            const solutionsText = (card.sample_solutions || []).join(' · ');
            return `
              <div class="didactic-card ${isSelected ? 'selected-card' : ''}" data-card-num="${card.number}">
                <div class="didactic-card-inner">
                  <div class="card-top-meta">
                    <span class="card-num-circle">Tarjeta #${card.number}</span>
                    <span class="card-kind-tag kind-${escapeHtml(card.kind_id)}">${escapeHtml(card.kind_label)}</span>
                  </div>
                  <p class="card-prompt-text">${escapeHtml(card.prompt)}</p>
                  <div class="card-bottom-bar">
                    <span class="card-solutions-preview" title="${escapeHtml(solutionsText)}">
                      ${
                        showSolutions
                          ? `<strong>Resp:</strong> ${escapeHtml(solutionsText)}`
                          : '<em>Soluciones ocultas</em>'
                      }
                    </span>
                    <button type="button" class="locate-in-cloud-btn" data-card-num="${card.number}">
                      ${hasWordCloud ? '🔍 Localizar en la hoja' : '📄 Ver Lámina PDF'}
                    </button>
                  </div>
                </div>
              </div>
            `;
          })
          .join('');

        return `
          <div class="card-sheet-page">
            <div class="sheet-page-label">
              <span>Bloque ${pageIdx + 1} de ${chunks.length} (${escapeHtml(preview.paper_size_label)})</span>
              <span>Ítems ${pageIdx * 6 + 1} a ${pageIdx * 6 + chunk.length}</span>
            </div>
            <div class="cards-grid-3x2">
              ${cardsHtml}
            </div>
          </div>
        `;
      })
      .join('');

    els.cardsPagesContainer.querySelectorAll('.locate-in-cloud-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const cardNum = Number(btn.dataset.cardNum);
        if (hasWordCloud) {
          highlightCardInWordCloud(cardNum, true);
        } else {
          switchStudioTab('pdf');
        }
      });
    });
  }

  // =========================================================================
  // PESTAÑA 2: NUBE DE PALABRAS INTERACTIVA (SVG) E INSPECTOR LÉXICO
  // =========================================================================

  function renderWordCloudControls(preview) {
    const cards = preview.cards || [];
    els.wcCardHighlightSelect.innerHTML =
      `<option value="">— Ninguna tarjeta seleccionada —</option>` +
      cards
        .map(
          (c) =>
            `<option value="${c.number}">#${c.number}: ${escapeHtml(
              c.prompt.length > 58 ? c.prompt.slice(0, 58) + '…' : c.prompt
            )}</option>`
        )
        .join('');

    const cats = (preview.summary && preview.summary.categories) || [];
    els.wcCategoryFilterSelect.innerHTML =
      `<option value="">Todas las categorías (${preview.placed_words.length})</option>` +
      cats
        .map(
          (c) => `<option value="${escapeHtml(c.label)}">${escapeHtml(c.label)} (${c.count})</option>`
        )
        .join('');
  }

  function renderWordCloudSvg(preview) {
    const pageW = preview.page_width_pt || 841.89;
    const pageH = preview.page_height_pt || 595.28;
    const words = preview.placed_words || [];

    els.wordcloudSvg.setAttribute('viewBox', `0 0 ${pageW} ${pageH}`);

    const borderMargin = 16;
    const borderW = pageW - borderMargin * 2;
    const borderH = pageH - borderMargin * 2;

    const wordsSvg = words
      .map((pw) => {
        const scaleX = (pw.horiz_scale || 100) / 100;
        const fillStyle = pw.outlined ? 'none' : '#14171c';
        const strokeStyle = pw.outlined ? '#1e2026' : 'none';
        const strokeWidth = pw.outlined ? '0.9' : '0';
        const padX = 5;
        const padY = 3;

        return `
          <g class="svg-word-group"
             data-word-index="${pw.index}"
             data-category="${escapeHtml(pw.category_label)}"
             transform="translate(${pw.svg_x.toFixed(2)}, ${pw.svg_y.toFixed(2)}) rotate(${pw.svg_angle_deg.toFixed(2)})">
            <rect class="word-highlight-box"
                  x="${(-pw.width / 2 - padX).toFixed(2)}"
                  y="${(-pw.height / 2 - padY).toFixed(2)}"
                  width="${(pw.width + padX * 2).toFixed(2)}"
                  height="${(pw.height + padY * 2).toFixed(2)}"
                  rx="5"
                  fill="transparent"
                  stroke="none" />
            <text x="0"
                  y="${(pw.height * 0.34).toFixed(2)}"
                  text-anchor="middle"
                  transform="scale(${scaleX.toFixed(3)}, 1)"
                  font-family="${escapeHtml(pw.font_family_css)}"
                  font-weight="${escapeHtml(pw.font_weight_css)}"
                  font-style="${escapeHtml(pw.font_style_css)}"
                  font-size="${pw.font_size.toFixed(2)}"
                  fill="${fillStyle}"
                  stroke="${strokeStyle}"
                  stroke-width="${strokeWidth}">
              ${escapeHtml(pw.display_text)}
            </text>
          </g>
        `;
      })
      .join('');

    els.wordcloudSvg.innerHTML = `
      <rect x="0" y="0" width="${pageW}" height="${pageH}" fill="#ffffff" />
      <rect x="${borderMargin}" y="${borderMargin}" width="${borderW}" height="${borderH}"
            rx="10" fill="none" stroke="#b8bdc4" stroke-width="1.4" />
      ${wordsSvg}
    `;

    els.wordcloudSvg.querySelectorAll('.svg-word-group').forEach((grp) => {
      grp.addEventListener('click', (ev) => {
        ev.stopPropagation();
        const idx = Number(grp.dataset.wordIndex);
        selectWordInInspector(idx);
      });
    });

    applyWordCloudHighlights();
  }

  function highlightCardInWordCloud(cardNumber, switchTab = false) {
    state.selectedCardNumber = cardNumber;
    state.selectedCategoryLabel = '';
    els.wcCardHighlightSelect.value = cardNumber ? String(cardNumber) : '';
    els.wcCategoryFilterSelect.value = '';

    if (switchTab) {
      switchStudioTab('wordcloud');
    }

    applyWordCloudHighlights();
    updateSelectionBanner();
    renderCardsPreview(state.currentPreview);
  }

  function applyWordCloudHighlights() {
    if (!state.currentPreview) return;

    const groups = els.wordcloudSvg.querySelectorAll('.svg-word-group');
    let highlightedIndices = new Set();
    let hasActiveFilter = false;

    if (state.selectedCardNumber) {
      const card = (state.currentPreview.cards || []).find(
        (c) => c.number === state.selectedCardNumber
      );
      if (card) {
        hasActiveFilter = true;
        (card.matched_word_indices || []).forEach((idx) => highlightedIndices.add(idx));
      }
    } else if (state.selectedCategoryLabel) {
      hasActiveFilter = true;
      (state.currentPreview.placed_words || []).forEach((pw) => {
        if (pw.category_label === state.selectedCategoryLabel) {
          highlightedIndices.add(pw.index);
        }
      });
    }

    groups.forEach((grp) => {
      const idx = Number(grp.dataset.wordIndex);
      const isSolution = highlightedIndices.has(idx);
      const isSelectedWord = state.selectedWordIndex === idx;

      grp.classList.toggle('highlighted-solution', isSolution);
      grp.classList.toggle('selected-word', isSelectedWord);
      grp.classList.toggle('dimmed', hasActiveFilter && !isSolution && !isSelectedWord);
    });
  }

  function selectWordInInspector(wordIndex) {
    if (!state.currentPreview) return;
    const pw = (state.currentPreview.placed_words || []).find((w) => w.index === wordIndex);
    if (!pw) return;

    state.selectedWordIndex = wordIndex;
    applyWordCloudHighlights();

    els.inspectorEmptyHint.hidden = true;
    els.inspectorDetails.hidden = false;

    els.inspWord.textContent = pw.word;
    els.inspCategory.textContent = pw.category_label;
    els.inspSyllables.textContent = String(pw.syllables);
    els.inspLetters.textContent = String(pw.letters);
    els.inspVowels.textContent = String(pw.vowels);
    els.inspConsonants.textContent = String(pw.consonants);

    // Sinónimos en la hoja
    if (pw.synonyms_on_sheet && pw.synonyms_on_sheet.length > 0) {
      els.inspSynonyms.innerHTML = pw.synonyms_on_sheet
        .map((s) => `<span class="rel-pill" data-jump-word="${escapeHtml(s)}">${escapeHtml(s)}</span>`)
        .join('');
    } else {
      els.inspSynonyms.textContent = 'Ninguno en esta hoja';
    }

    // Antónimos en la hoja
    if (pw.antonyms_on_sheet && pw.antonyms_on_sheet.length > 0) {
      els.inspAntonyms.innerHTML = pw.antonyms_on_sheet
        .map((a) => `<span class="rel-pill" data-jump-word="${escapeHtml(a)}">${escapeHtml(a)}</span>`)
        .join('');
    } else {
      els.inspAntonyms.textContent = 'Ninguno en esta hoja';
    }

    // Tarjetas que aceptan esta palabra como solución
    const matchingCards = (state.currentPreview.cards || []).filter((c) =>
      (c.matched_word_indices || []).includes(wordIndex)
    );

    if (matchingCards.length > 0) {
      els.inspMatchingCards.innerHTML = matchingCards
        .map(
          (c) => `
          <div class="rel-card-item" data-card-num="${c.number}">
            <strong>#${c.number}:</strong> ${escapeHtml(c.prompt)}
          </div>
        `
        )
        .join('');
    } else {
      els.inspMatchingCards.innerHTML = `<span style="font-size:0.76rem;color:var(--text-muted);">Palabra de contexto en la hoja</span>`;
    }

    // Eventos para saltar a sinónimo/antónimo o resaltar tarjeta
    els.inspectorDetails.querySelectorAll('[data-jump-word]').forEach((pill) => {
      pill.addEventListener('click', () => {
        const targetWord = pill.dataset.jumpWord;
        const targetObj = (state.currentPreview.placed_words || []).find(
          (w) => w.word === targetWord
        );
        if (targetObj) {
          selectWordInInspector(targetObj.index);
        }
      });
    });

    els.inspectorDetails.querySelectorAll('[data-card-num]').forEach((item) => {
      item.addEventListener('click', () => {
        highlightCardInWordCloud(Number(item.dataset.cardNum), false);
      });
    });
  }

  function updateSelectionBanner() {
    if (state.selectedCardNumber && state.currentPreview) {
      const card = (state.currentPreview.cards || []).find(
        (c) => c.number === state.selectedCardNumber
      );
      if (card) {
        els.activeSelectionBanner.hidden = false;
        els.activeSelectionText.textContent = `Resaltando respuestas de Tarjeta #${card.number} (${card.sample_solutions.length} en la hoja)`;
        return;
      }
    }
    if (state.selectedCategoryLabel) {
      els.activeSelectionBanner.hidden = false;
      els.activeSelectionText.textContent = `Filtrando categoría: ${state.selectedCategoryLabel}`;
      return;
    }
    els.activeSelectionBanner.hidden = true;
  }

  // =========================================================================
  // PESTAÑA 3: HOJA DE RESPUESTAS PARA EL DOCENTE (SOLUCIONARIO)
  // =========================================================================

  function renderTeacherSolutionsTable(preview) {
    if (!preview) return;
    const query = (els.teacherSearchInput.value || '').trim().toLowerCase();
    const cards = (preview.cards || []).filter((c) => {
      if (!query) return true;
      const text = `${c.prompt} ${c.kind_label} ${(c.sample_solutions || []).join(' ')}`.toLowerCase();
      return text.includes(query);
    });

    els.teacherSolutionsTbody.innerHTML = cards
      .map((card) => {
        const badges = (card.sample_solutions || [])
          .map(
            (sol) =>
              `<span class="solution-word-badge" data-sol-word="${escapeHtml(sol)}">${escapeHtml(
                sol
              )}</span>`
          )
          .join('');

        return `
          <tr>
            <td><strong>#${card.number}</strong></td>
            <td><span class="card-kind-tag kind-${escapeHtml(card.kind_id)}">${escapeHtml(card.kind_label)}</span></td>
            <td>${escapeHtml(card.prompt)}</td>
            <td>${badges}</td>
            <td>
              <button type="button" class="mini-btn teacher-locate-btn" data-card-num="${card.number}">
                Ver en Nube
              </button>
            </td>
          </tr>
        `;
      })
      .join('');

    els.teacherSolutionsTbody.querySelectorAll('.teacher-locate-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        highlightCardInWordCloud(Number(btn.dataset.cardNum), true);
      });
    });

    els.teacherSolutionsTbody.querySelectorAll('.solution-word-badge').forEach((badge) => {
      badge.addEventListener('click', () => {
        const wordStr = badge.dataset.solWord;
        const pw = (state.currentPreview.placed_words || []).find((w) => w.word === wordStr);
        if (pw) {
          switchStudioTab('wordcloud');
          selectWordInInspector(pw.index);
        }
      });
    });
  }

  // =========================================================================
  // PESTAÑA 4: VISOR PDF Y EXPORTACIÓN
  // =========================================================================

  function buildPdfEndpointUrl(saveToLibrary = false, download = false) {
    const params = collectQueryParams(false);
    params.set('save', saveToLibrary ? '1' : '0');
    if (download) {
      params.set('download', '1');
    }
    return `/api/tools/${encodeURIComponent(state.activeToolId)}/pdf?${params.toString()}`;
  }

  function refreshPdfIframe() {
    const url = buildPdfEndpointUrl(false, false);
    els.pdfPreviewIframe.src = url;
    els.directPdfOpenLink.href = url;
  }

  async function saveCurrentPdfToLibrary() {
    try {
      const params = collectQueryParams(false);
      const res = await fetch(
        `/api/tools/${encodeURIComponent(state.activeToolId)}/save?${params.toString()}`,
        { method: 'POST' }
      );
      if (!res.ok) throw new Error('Error al guardar en biblioteca');
      const data = await res.json();
      showToast(`Guardado en biblioteca: ${data.filename}`);
      await loadMaterialsLibrary();
    } catch (err) {
      console.error(err);
      showToast('Error al guardar el PDF en la biblioteca.');
    }
  }

  // =========================================================================
  // VISTA 3: BIBLIOTECA DE MATERIALES GENERADOS
  // =========================================================================

  async function loadMaterialsLibrary() {
    try {
      const res = await fetch('/api/materials');
      if (!res.ok) return;
      const data = await res.json();
      state.materials = data.materials || [];

      els.libraryCountBadge.textContent = String(state.materials.length);
      els.statPdfsCount.textContent = String(state.materials.length);

      renderMaterialsTable();
    } catch (err) {
      console.error(err);
    }
  }

  function renderMaterialsTable() {
    if (state.materials.length === 0) {
      els.libraryFilesTbody.innerHTML = `
        <tr>
          <td colspan="4" style="text-align:center;color:var(--text-muted);">
            Aún no hay archivos PDF guardados. Genera uno desde el Estudio.
          </td>
        </tr>
      `;
      return;
    }

    els.libraryFilesTbody.innerHTML = state.materials
      .map((mat) => {
        const fileUrl = `/api/materials/${encodeURIComponent(mat.filename)}`;
        return `
          <tr>
            <td><strong>${escapeHtml(mat.filename)}</strong></td>
            <td><span class="badge-chip">${escapeHtml(mat.tool_id)}</span></td>
            <td>${formatBytes(mat.size_bytes)}</td>
            <td style="display:flex;gap:0.4rem;flex-wrap:wrap;">
              <button type="button" class="mini-btn preview-lib-btn" data-filename="${escapeHtml(mat.filename)}">
                👁️ Ver
              </button>
              <a href="${fileUrl}?download=1" download="${escapeHtml(mat.filename)}" class="mini-btn">
                ⬇ Descargar
              </a>
              ${
                mat.deletable
                  ? `<button type="button" class="mini-btn delete-lib-btn" data-filename="${escapeHtml(mat.filename)}" style="color:var(--accent-rose);">🗑️</button>`
                  : ''
              }
            </td>
          </tr>
        `;
      })
      .join('');

    els.libraryFilesTbody.querySelectorAll('.preview-lib-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        previewLibraryFile(btn.dataset.filename);
      });
    });

    els.libraryFilesTbody.querySelectorAll('.delete-lib-btn').forEach((btn) => {
      btn.addEventListener('click', async () => {
        const filename = btn.dataset.filename;
        await fetch(`/api/materials/${encodeURIComponent(filename)}`, { method: 'DELETE' });
        showToast(`Eliminado: ${filename}`);
        loadMaterialsLibrary();
      });
    });

    // Cargar el primero por defecto en el visor si está vacío
    if (!els.libraryPdfIframe.getAttribute('src') && state.materials.length > 0) {
      previewLibraryFile(state.materials[0].filename);
    }
  }

  function previewLibraryFile(filename) {
    const url = `/api/materials/${encodeURIComponent(filename)}`;
    els.libraryViewerTitle.textContent = filename;
    els.libraryPdfIframe.src = url;
    els.libraryViewerDownload.href = `${url}?download=1`;
    els.libraryViewerDownload.download = filename;
    els.libraryViewerDownload.hidden = false;
  }

  // =========================================================================
  // EVENTOS E INICIALIZACIÓN
  // =========================================================================

  function initEvents() {
    // Navegación principal
    els.navBtns.forEach((btn) => {
      btn.addEventListener('click', () => switchView(btn.dataset.view));
    });

    // Pestañas del Estudio
    els.previewTabs.forEach((tab) => {
      tab.addEventListener('click', () => switchStudioTab(tab.dataset.tab));
    });

    // Tema claro / oscuro
    els.themeToggleBtn.addEventListener('click', () => {
      const html = document.documentElement;
      const next = html.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
      html.setAttribute('data-theme', next);
    });

    // Buscador del catálogo
    els.catalogSearchInput.addEventListener('input', renderCatalogGrid);

    // Sincronizar comunicador
    els.rescanToolsBtn.addEventListener('click', async () => {
      await fetch('/api/communicator/rescan', { method: 'POST' });
      await loadCommunicatorCatalog();
      showToast('Comunicador sincronizado con éxito.');
    });

    // Selector de herramienta en el estudio
    els.studioToolSelector.addEventListener('change', () => {
      selectToolInStudio(els.studioToolSelector.value);
    });

    // Nueva semilla aleatoria
    els.randomSeedBtn.addEventListener('click', () => {
      fetchLivePreview(true);
      showToast('Nueva variación generada en vivo.');
    });

    // Guardar y descargar PDF
    els.generateDownloadBtn.addEventListener('click', () => {
      const url = buildPdfEndpointUrl(true, true);
      window.open(url, '_blank');
      setTimeout(loadMaterialsLibrary, 600);
      showToast('PDF generado, descargado y guardado en la biblioteca.');
    });

    // Guardar solo en biblioteca
    els.saveLibraryOnlyBtn.addEventListener('click', saveCurrentPdfToLibrary);

    // Mostrar/ocultar soluciones en tarjetas
    els.toggleCardSolutions.addEventListener('change', () => {
      if (state.currentPreview) renderCardsPreview(state.currentPreview);
    });

    // Limpiar resaltado
    els.clearHighlightBtn.addEventListener('click', () => {
      state.selectedCardNumber = null;
      state.selectedCategoryLabel = '';
      state.selectedWordIndex = null;
      els.wcCardHighlightSelect.value = '';
      els.wcCategoryFilterSelect.value = '';
      applyWordCloudHighlights();
      updateSelectionBanner();
      if (state.currentPreview) {
        renderCardsPreview(state.currentPreview);
        renderStudioSummary(state.currentPreview);
      }
    });

    // Filtros en Nube de Palabras
    els.wcCardHighlightSelect.addEventListener('change', () => {
      const val = els.wcCardHighlightSelect.value;
      highlightCardInWordCloud(val ? Number(val) : null, false);
    });

    els.wcCategoryFilterSelect.addEventListener('change', () => {
      state.selectedCategoryLabel = els.wcCategoryFilterSelect.value;
      state.selectedCardNumber = null;
      els.wcCardHighlightSelect.value = '';
      applyWordCloudHighlights();
      updateSelectionBanner();
      if (state.currentPreview) renderStudioSummary(state.currentPreview);
    });

    // Zoom en Nube de Palabras
    const updateZoom = (newZoom) => {
      state.zoomLevel = Math.max(60, Math.min(180, newZoom));
      els.wcZoomLabel.textContent = `${state.zoomLevel}%`;
      els.svgSheetWrapper.style.maxWidth = `${Math.round((880 * state.zoomLevel) / 100)}px`;
    };
    els.wcZoomOutBtn.addEventListener('click', () => updateZoom(state.zoomLevel - 15));
    els.wcZoomInBtn.addEventListener('click', () => updateZoom(state.zoomLevel + 15));
    els.wcZoomResetBtn.addEventListener('click', () => updateZoom(100));

    // Solucionario Docente
    els.teacherSearchInput.addEventListener('input', () => {
      renderTeacherSolutionsTable(state.currentPreview);
    });
    els.printTeacherKeyBtn.addEventListener('click', () => window.print());

    // Biblioteca
    els.refreshLibraryBtn.addEventListener('click', loadMaterialsLibrary);
  }

  // Arranque
  initEvents();
  loadCommunicatorCatalog();
  loadMaterialsLibrary();
})();
