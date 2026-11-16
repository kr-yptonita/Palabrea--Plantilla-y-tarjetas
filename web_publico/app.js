(() => {
  'use strict';

  const state = {
    presets: [],
    selectedGrade: 'all',
    selectedSubject: 'all',
    paperFormat: 'a4',
    searchQuery: '',
    presetSeeds: {},
    presetNotations: {}, // 'tradicional' | 'galera' por cada preset de división
    activeModalPreset: null,
  };

  const els = {
    paperBtns: document.querySelectorAll('.paper-btn'),
    themeBtn: document.getElementById('pub-theme-btn'),
    brandResetLink: document.getElementById('brand-reset-link'),
    edNavLinks: document.querySelectorAll('.ed-nav-link'),
    heroTiles: document.querySelectorAll('.hero-tile'),
    peachCols: document.querySelectorAll('.peach-col'),
    gradeCards: document.querySelectorAll('.grade-card'),
    subjectPills: document.querySelectorAll('.subject-pill'),
    resetFiltersBtn: document.getElementById('reset-filters-btn'),
    searchInput: document.getElementById('pub-search-input'),
    catalogSection: document.getElementById('catalogo-section'),
    activeCollectionHeading: document.getElementById('active-collection-heading'),
    resultsCountText: document.getElementById('results-count-text'),
    presetsGrid: document.getElementById('presets-grid'),

    // Modal
    previewModal: document.getElementById('preview-modal'),
    modalPresetGrade: document.getElementById('modal-preset-grade'),
    modalPresetTitle: document.getElementById('modal-preset-title'),
    modalPresetSubtitle: document.getElementById('modal-preset-subtitle'),
    modalCloseBtn: document.getElementById('modal-close-btn'),
    modalNewVariationBtn: document.getElementById('modal-new-variation-btn'),
    modalDivNotationBox: document.getElementById('modal-div-notation-box'),
    modalDownloadLink: document.getElementById('modal-download-link'),
    modalPdfIframe: document.getElementById('modal-pdf-iframe'),
    modalSamplesList: document.getElementById('modal-samples-list'),

    toast: document.getElementById('pub-toast'),
  };

  let toastTimer = null;
  function showToast(msg) {
    if (!els.toast) return;
    els.toast.textContent = msg;
    els.toast.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      els.toast.hidden = true;
    }, 3000);
  }

  function escapeHtml(str) {
    return String(str ?? '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function presetHasDivision(preset) {
    const q = preset.query_string || '';
    return q.includes('operation=division') || q.includes('operation=mixto');
  }

  function getPresetNotation(preset) {
    if (state.presetNotations[preset.id]) {
      return state.presetNotations[preset.id];
    }
    const params = new URLSearchParams(preset.query_string || '');
    const initial = params.get('notation') === 'galera' ? 'galera' : 'tradicional';
    state.presetNotations[preset.id] = initial;
    return initial;
  }

  function getOrInitSeed(presetId, forceNew = false) {
    if (forceNew || !state.presetSeeds[presetId]) {
      state.presetSeeds[presetId] = Math.floor(100000 + Math.random() * 8999999);
    }
    return state.presetSeeds[presetId];
  }

  function buildPresetUrl(preset, endpoint, options = {}) {
    const params = new URLSearchParams(preset.query_string || '');
    params.set('format', state.paperFormat);

    if (presetHasDivision(preset)) {
      const notation = getPresetNotation(preset);
      params.set('notation', notation);
      const baseFile = (params.get('filename') || 'ficha_divisiones.pdf').replace(/\.pdf$/i, '');
      params.set('filename', `${baseFile}_${notation}.pdf`);
    }

    const seed = getOrInitSeed(preset.id, Boolean(options.newVariation));
    params.set('seed', String(seed));

    if (options.save) {
      params.set('save', '1');
    }
    if (options.download) {
      params.set('download', '1');
    }

    return `/api/tools/${encodeURIComponent(preset.tool_id)}/${endpoint}?${params.toString()}`;
  }

  async function loadPublicPresets() {
    try {
      const res = await fetch('/api/presets');
      if (!res.ok) throw new Error('Error al cargar /api/presets');
      const data = await res.json();
      state.presets = data.presets || [];
      renderPresetsGrid();
    } catch (err) {
      console.error(err);
      els.presetsGrid.innerHTML = `
        <div class="step-box">
          <h4>No se pudo conectar con el servidor de materiales</h4>
          <p>Verifica que el servidor esté activo e intenta recargar la página.</p>
        </div>
      `;
    }
  }

  function getCoverDecorText(preset) {
    const q = (preset.query_string || '').toLowerCase();
    if (q.includes('operation=division')) {
      return getPresetNotation(preset) === 'galera' ? '4 ⟌ 84  ·  12 ⟌ 360' : '84 | 4  ·  360 | 12';
    }
    if (q.includes('operation=multiplicacion')) return '34 × 12  ·  128 × 6';
    if (q.includes('operation=suma')) return '425 + 189  ·  84 + 27';
    if (q.includes('operation=resta')) return '704 − 268  ·  95 − 37';
    if (q.includes('operation=mixto')) return '24 + 18  ·  84 ÷ 4';
    if (preset.subject_id === 'lectura') return 'El _____ corre por el parque.';
    if (preset.subject_id === 'palabras') return 'MARIPOSA  ·  RELÁMPAGO';
    return 'PDF';
  }

  function updateCollectionHeading() {
    if (!els.activeCollectionHeading) return;
    const gradeLabels = {
      all: '',
      ciclo1: '1.º Y 2.º PRIMARIA',
      ciclo2: '3.º Y 4.º PRIMARIA',
      ciclo3: '5.º Y 6.º PRIMARIA',
    };
    const subjectLabels = {
      all: '',
      matematicas: 'MATEMÁTICAS',
      lectura: 'LECTURA Y ESCRITURA',
      palabras: 'VOCABULARIO',
      featured: 'DESTACADOS',
    };

    const g = gradeLabels[state.selectedGrade] || '';
    const s = subjectLabels[state.selectedSubject] || '';

    if (g && s) {
      els.activeCollectionHeading.textContent = `COLECCIÓN ${s} · ${g}`;
    } else if (g) {
      els.activeCollectionHeading.textContent = `COLECCIÓN ${g}`;
    } else if (s) {
      els.activeCollectionHeading.textContent = `COLECCIÓN DE ${s}`;
    } else {
      els.activeCollectionHeading.textContent = 'COLECCIÓN COMPLETA DE MATERIALES';
    }
  }

  function setGradeFilter(gradeId, shouldScroll = false) {
    state.selectedGrade = gradeId;
    els.gradeCards.forEach((c) => c.classList.toggle('active', c.dataset.grade === gradeId));
    els.peachCols.forEach((c) => c.classList.toggle('active', c.dataset.grade === gradeId));
    updateCollectionHeading();
    renderPresetsGrid();
    if (shouldScroll && els.catalogSection) {
      els.catalogSection.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  function setSubjectFilter(subjectId, shouldScroll = false) {
    state.selectedSubject = subjectId;
    els.subjectPills.forEach((p) => p.classList.toggle('active', p.dataset.subject === subjectId));
    els.edNavLinks.forEach((l) => l.classList.toggle('active', l.dataset.subject === subjectId));
    updateCollectionHeading();
    renderPresetsGrid();
    if (shouldScroll && els.catalogSection) {
      els.catalogSection.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  function resetAllFilters() {
    state.selectedGrade = 'all';
    state.selectedSubject = 'all';
    state.searchQuery = '';
    if (els.searchInput) els.searchInput.value = '';
    els.gradeCards.forEach((c) => c.classList.toggle('active', c.dataset.grade === 'all'));
    els.peachCols.forEach((c) => c.classList.remove('active'));
    els.subjectPills.forEach((p) => p.classList.toggle('active', p.dataset.subject === 'all'));
    els.edNavLinks.forEach((l) => l.classList.toggle('active', l.dataset.subject === 'all'));
    updateCollectionHeading();
    renderPresetsGrid();
  }

  function renderPresetsGrid() {
    const query = state.searchQuery.trim().toLowerCase();

    const filtered = state.presets.filter((p) => {
      if (state.selectedGrade !== 'all' && p.grade_id !== state.selectedGrade) {
        return false;
      }
      if (state.selectedSubject === 'featured') {
        if (!p.featured) return false;
      } else if (state.selectedSubject !== 'all' && p.subject_id !== state.selectedSubject) {
        return false;
      }
      if (query) {
        const text = `${p.title} ${p.subtitle} ${p.description} ${p.grade_label} ${p.subject_label}`.toLowerCase();
        if (!text.includes(query)) return false;
      }
      return true;
    });

    const paperLabel = state.paperFormat === 'letter' ? 'Carta' : 'A4';
    els.resultsCountText.textContent = `Mostrando ${filtered.length} ${
      filtered.length === 1 ? 'ficha gratuita lista' : 'fichas gratuitas listas'
    } para imprimir en tamaño ${paperLabel === 'Carta' ? 'Carta (Letter)' : 'A4'}`;

    if (filtered.length === 0) {
      els.presetsGrid.innerHTML = `
        <div class="step-box" style="grid-column: 1 / -1; text-align: center; padding: 2.5rem 1.5rem;">
          <h4>No encontramos fichas con ese filtro</h4>
          <p>Prueba seleccionando "Todos los Grados" o "Todas las áreas".</p>
        </div>
      `;
      return;
    }

    els.presetsGrid.innerHTML = filtered
      .map((preset) => {
        const isDiv = presetHasDivision(preset);
        const currentNotation = isDiv ? getPresetNotation(preset) : null;
        const coverDecor = getCoverDecorText(preset);
        const divSelectorHtml = isDiv
          ? `
            <div class="division-options-box" role="group" aria-label="Elige el formato de la división">
              <span class="division-options-title">Formato de división (2 opciones):</span>
              <div class="division-options-grid">
                <button type="button"
                        class="notation-opt-btn card-notation-btn ${
                          currentNotation === 'tradicional' ? 'active' : ''
                        }"
                        data-preset-id="${escapeHtml(preset.id)}"
                        data-notation="tradicional">
                  Caja (84 | 4)
                </button>
                <button type="button"
                        class="notation-opt-btn card-notation-btn ${
                          currentNotation === 'galera' ? 'active' : ''
                        }"
                        data-preset-id="${escapeHtml(preset.id)}"
                        data-notation="galera">
                  Casita (4 ⟌ 84)
                </button>
              </div>
            </div>
          `
          : '';

        const notationBadgeText = isDiv
          ? currentNotation === 'galera'
            ? ' · Casita ⟌'
            : ' · Caja |─'
          : '';

        return `
        <article class="preset-card theme-${escapeHtml(preset.color_theme)}">
          <div>
            <div class="preset-visual-cover"
                 data-id="${escapeHtml(preset.id)}"
                 role="button"
                 tabindex="0"
                 aria-label="Ver vista previa de ${escapeHtml(preset.title)}">
              <div class="cover-worksheet-preview" aria-hidden="true">${escapeHtml(coverDecor)}</div>
              <div class="cover-overlay" aria-hidden="true"></div>
              <div class="cover-top-badges">
                <span class="cover-grade-pill">${escapeHtml(preset.grade_label)}</span>
                <span class="cover-free-pill">GRATIS</span>
              </div>
              <h3 class="cover-center-title">${escapeHtml(preset.title)}</h3>
            </div>

            <div class="preset-body">
              <p class="preset-subtitle">${escapeHtml(preset.subject_label)} · ${escapeHtml(preset.subtitle)}</p>
              <p class="preset-desc">${escapeHtml(preset.description)}</p>
              ${divSelectorHtml}
              <div class="preset-includes-box">
                <span>✓</span>
                <span>${escapeHtml(preset.includes_summary)}</span>
              </div>
            </div>
          </div>

          <div class="preset-actions">
            <div class="download-split-box">
              <button type="button" class="pub-btn primary download-preset-btn" data-id="${escapeHtml(preset.id)}">
                Descargar PDF (${paperLabel}${notationBadgeText})
              </button>
              <div class="card-paper-switch" role="group" aria-label="Cambiar tamaño de hoja">
                <button type="button"
                        class="card-paper-btn ${state.paperFormat === 'a4' ? 'active' : ''}"
                        data-paper="a4"
                        title="Cambiar a tamaño A4">
                  A4
                </button>
                <button type="button"
                        class="card-paper-btn ${state.paperFormat === 'letter' ? 'active' : ''}"
                        data-paper="letter"
                        title="Cambiar a tamaño Carta (Letter)">
                  Carta
                </button>
              </div>
            </div>
            <div class="preset-secondary-row">
              <button type="button" class="pub-btn secondary preview-preset-btn" data-id="${escapeHtml(preset.id)}">
                Vista Previa
              </button>
              <button type="button" class="pub-btn secondary another-preset-btn" data-id="${escapeHtml(preset.id)}" title="Genera una ficha completamente diferente de este mismo tema">
                Generar otra
              </button>
            </div>
          </div>
        </article>
      `;
      })
      .join('');

    // Clic en la portada visual para abrir vista previa
    els.presetsGrid.querySelectorAll('.preset-visual-cover').forEach((cover) => {
      const openHandler = () => {
        const preset = state.presets.find((p) => p.id === cover.dataset.id);
        if (preset) openPresetPreviewModal(preset, false);
      };
      cover.addEventListener('click', openHandler);
      cover.addEventListener('keydown', (ev) => {
        if (ev.key === 'Enter' || ev.key === ' ') {
          ev.preventDefault();
          openHandler();
        }
      });
    });

    // Eventos del selector de las 2 opciones de división en cada tarjeta
    els.presetsGrid.querySelectorAll('.card-notation-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const presetId = btn.dataset.presetId;
        const notation = btn.dataset.notation;
        state.presetNotations[presetId] = notation;
        renderPresetsGrid();
        showToast(
          notation === 'galera'
            ? 'Seleccionado: División en Casita / Galera (4 ⟌ 84)'
            : 'Seleccionado: División en Caja Tradicional (84 | 4)'
        );
      });
    });

    // Selector de tamaño de hoja junto al botón de descarga en cada tarjeta
    els.presetsGrid.querySelectorAll('.card-paper-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        setPaperFormat(btn.dataset.paper);
      });
    });

    // Eventos de descarga, vista previa y variación
    els.presetsGrid.querySelectorAll('.download-preset-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const preset = state.presets.find((p) => p.id === btn.dataset.id);
        if (!preset) return;
        const url = buildPresetUrl(preset, 'pdf', { save: true, download: true });
        window.open(url, '_blank');
        showToast(`¡Descargando "${preset.title}"!`);
      });
    });

    els.presetsGrid.querySelectorAll('.preview-preset-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const preset = state.presets.find((p) => p.id === btn.dataset.id);
        if (!preset) return;
        openPresetPreviewModal(preset, false);
      });
    });

    els.presetsGrid.querySelectorAll('.another-preset-btn').forEach((btn) => {
      btn.addEventListener('click', () => {
        const preset = state.presets.find((p) => p.id === btn.dataset.id);
        if (!preset) return;
        const url = buildPresetUrl(preset, 'pdf', {
          newVariation: true,
          save: true,
          download: true,
        });
        window.open(url, '_blank');
        showToast(`Nueva ficha diferente de "${preset.title}" generada.`);
      });
    });
  }

  function syncModalNotationButtons(preset) {
    if (!els.modalDivNotationBox) return;
    if (!presetHasDivision(preset)) {
      els.modalDivNotationBox.hidden = true;
      return;
    }
    els.modalDivNotationBox.hidden = false;
    const activeNotation = getPresetNotation(preset);
    els.modalDivNotationBox.querySelectorAll('.notation-opt-btn').forEach((btn) => {
      btn.classList.toggle('active', btn.dataset.notation === activeNotation);
    });
  }

  async function openPresetPreviewModal(preset, forceNewVariation = false) {
    state.activeModalPreset = preset;
    els.modalPresetGrade.textContent = `${preset.grade_label} · ${preset.subject_label}`;
    els.modalPresetTitle.textContent = preset.title;
    els.modalPresetSubtitle.textContent = preset.subtitle;

    syncModalNotationButtons(preset);

    const pdfInlineUrl = buildPresetUrl(preset, 'pdf', {
      newVariation: forceNewVariation,
      save: false,
      download: false,
    });
    const pdfDownloadUrl = buildPresetUrl(preset, 'pdf', {
      newVariation: false,
      save: true,
      download: true,
    });

    const paperLabel = state.paperFormat === 'letter' ? 'Carta' : 'A4';
    els.modalPdfIframe.src = pdfInlineUrl;
    els.modalDownloadLink.href = pdfDownloadUrl;
    els.modalDownloadLink.textContent = `Descargar PDF Gratis (${paperLabel})`;
    els.previewModal.hidden = false;

    // Cargar también el resumen de preguntas en el panel lateral del modal
    els.modalSamplesList.innerHTML = `<li class="sample-item">Cargando ejercicios de la ficha...</li>`;
    try {
      const previewUrl = buildPresetUrl(preset, 'preview', { newVariation: false });
      const res = await fetch(previewUrl);
      if (res.ok) {
        const preview = await res.json();
        const cards = preview.cards || [];
        els.modalSamplesList.innerHTML = cards
          .slice(0, 12)
          .map(
            (c) => `
            <li class="sample-item">
              <strong>#${c.number}:</strong> ${escapeHtml(c.prompt)}
              <span class="sample-ans">✓ ${escapeHtml((c.sample_solutions || []).join(' · '))}</span>
            </li>
          `
          )
          .join('');
      }
    } catch (e) {
      console.error(e);
    }
  }

  function closePreviewModal() {
    els.previewModal.hidden = true;
    els.modalPdfIframe.src = 'about:blank';
    state.activeModalPreset = null;
  }

  function setPaperFormat(nextFormat) {
    if (!nextFormat || (nextFormat !== 'a4' && nextFormat !== 'letter')) return;
    state.paperFormat = nextFormat;
    els.paperBtns.forEach((b) => b.classList.toggle('active', b.dataset.paper === nextFormat));
    renderPresetsGrid();
    if (state.activeModalPreset) {
      openPresetPreviewModal(state.activeModalPreset, false);
    }
    showToast(
      `Tamaño de hoja cambiado a ${
        state.paperFormat === 'letter' ? 'Carta (Letter)' : 'A4'
      }`
    );
  }

  function applyTheme(theme) {
    const next = theme === 'dark' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', next);
    if (els.themeBtn) {
      els.themeBtn.textContent = next === 'dark' ? 'MODO CLARO' : 'MODO OSCURO';
    }
  }

  function initTheme() {
    let saved = null;
    try {
      saved = localStorage.getItem('pub-theme');
    } catch (_) {}

    if (saved === 'dark' || saved === 'light') {
      applyTheme(saved);
    } else if (window.matchMedia) {
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      applyTheme(mq.matches ? 'dark' : 'light');
      const onChange = (ev) => {
        try {
          if (!localStorage.getItem('pub-theme')) {
            applyTheme(ev.matches ? 'dark' : 'light');
          }
        } catch (_) {
          applyTheme(ev.matches ? 'dark' : 'light');
        }
      };
      if (typeof mq.addEventListener === 'function') {
        mq.addEventListener('change', onChange);
      }
    }
  }

  function initAboutRotator() {
    const rotator = document.getElementById('about-rotator');
    if (!rotator) return;
    const slides = rotator.querySelectorAll('.rotator-slide');
    const dots = rotator.querySelectorAll('.rotator-dot');
    if (slides.length <= 1) return;

    let current = 0;
    setInterval(() => {
      slides[current].classList.remove('active');
      if (dots[current]) dots[current].classList.remove('active');
      current = (current + 1) % slides.length;
      slides[current].classList.add('active');
      if (dots[current]) dots[current].classList.add('active');
    }, 3600);
  }

  function initEvents() {
    if (window.location.pathname.replace(/\/+$/, '').endsWith('/trans')) {
      document.documentElement.setAttribute('data-easter', 'trans');
    }

    initTheme();
    initAboutRotator();

    // Selector de papel A4 / Carta (barra superior y modal)
    els.paperBtns.forEach((btn) => {
      btn.addEventListener('click', () => {
        setPaperFormat(btn.dataset.paper);
      });
    });

    // Modo claro / oscuro
    els.themeBtn.addEventListener('click', () => {
      const current = document.documentElement.getAttribute('data-theme');
      const next = current === 'dark' ? 'light' : 'dark';
      applyTheme(next);
      try {
        localStorage.setItem('pub-theme', next);
      } catch (_) {}
    });

    // Logo superior: restablece filtros
    if (els.brandResetLink) {
      els.brandResetLink.addEventListener('click', (ev) => {
        ev.preventDefault();
        resetAllFilters();
        window.scrollTo({ top: 0, behavior: 'smooth' });
      });
    }

    // Barra de navegación editorial superior
    els.edNavLinks.forEach((link) => {
      link.addEventListener('click', () => {
        setSubjectFilter(link.dataset.subject, true);
      });
    });

    // 3 Grandes tarjetas Hero
    els.heroTiles.forEach((tile) => {
      tile.addEventListener('click', () => {
        setSubjectFilter(tile.dataset.filterSubject, true);
      });
    });

    // 3 Columnas cálidas por ciclo de primaria
    els.peachCols.forEach((col) => {
      col.addEventListener('click', () => {
        setGradeFilter(col.dataset.grade, true);
      });
    });

    // Botón "Ver todas las colecciones ›"
    if (els.resetFiltersBtn) {
      els.resetFiltersBtn.addEventListener('click', () => {
        resetAllFilters();
        showToast('Mostrando todas las colecciones gratuitas');
      });
    }

    // Tarjetas de grado escolar
    els.gradeCards.forEach((card) => {
      card.addEventListener('click', () => {
        setGradeFilter(card.dataset.grade, false);
      });
    });

    // Píldoras de materia
    els.subjectPills.forEach((pill) => {
      pill.addEventListener('click', () => {
        setSubjectFilter(pill.dataset.subject, false);
      });
    });

    // Buscador
    els.searchInput.addEventListener('input', () => {
      state.searchQuery = els.searchInput.value || '';
      renderPresetsGrid();
    });

    // Selector de notación dentro del Modal
    if (els.modalDivNotationBox) {
      els.modalDivNotationBox.querySelectorAll('.notation-opt-btn').forEach((btn) => {
        btn.addEventListener('click', () => {
          if (!state.activeModalPreset) return;
          const notation = btn.dataset.notation;
          state.presetNotations[state.activeModalPreset.id] = notation;
          renderPresetsGrid();
          openPresetPreviewModal(state.activeModalPreset, false);
          showToast(
            notation === 'galera'
              ? 'Vista previa cambiada a Casita / Galera (4 ⟌ 84)'
              : 'Vista previa cambiada a Caja Tradicional (84 | 4)'
          );
        });
      });
    }

    // Modal
    els.modalCloseBtn.addEventListener('click', closePreviewModal);
    els.previewModal.addEventListener('click', (ev) => {
      if (ev.target === els.previewModal) {
        closePreviewModal();
      }
    });
    els.modalNewVariationBtn.addEventListener('click', () => {
      if (state.activeModalPreset) {
        openPresetPreviewModal(state.activeModalPreset, true);
        showToast('Se ha generado una hoja completamente diferente.');
      }
    });
  }

  initEvents();
  loadPublicPresets();
})();
