const createRustyMilkBindings = async () => {
  try {
    return await import('@rustymilk/web');
  } catch (importError) {
    const fallbackModule = await import('../../rustymilk-web/src/rustyMilkEngine.js');
    return fallbackModule;
  }
};

const webBindings = await createRustyMilkBindings();

const {
  createRustyMilkEngine,
  loadRustyMilkPack,
} = webBindings;

const REQUIRED_HOOKS = ['useCallback', 'useEffect', 'useMemo', 'useRef'];

const FRAME_DELAY_FALLBACK_MS = 16;

const resolveReact = (react = globalThis.React) => {
  const missing = REQUIRED_HOOKS.filter((name) => typeof react?.[name] !== 'function');
  if (missing.length) {
    throw new Error(
      `createRustyMilkReactBindings is missing required React hooks: ${missing.join(', ')}`,
    );
  }
  return react;
};

const clampFps = (value) => {
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) return 0;
  return Math.max(1, Math.floor(parsed));
};

const getFrameScheduler = () => {
  const hasWindow = typeof globalThis !== 'undefined';
  if (!hasWindow) {
    return {
      requestFrame: (callback) => setTimeout(() => callback(Date.now()), FRAME_DELAY_FALLBACK_MS),
      cancelFrame: (id) => clearTimeout(id),
    };
  }
  if (typeof globalThis.requestAnimationFrame === 'function') {
    return {
      requestFrame: globalThis.requestAnimationFrame.bind(globalThis),
      cancelFrame: globalThis.cancelAnimationFrame.bind(globalThis),
    };
  }
  return {
    requestFrame: (callback) => setTimeout(() => callback(Date.now()), FRAME_DELAY_FALLBACK_MS),
    cancelFrame: (id) => clearTimeout(id),
  };
};

export const createRustyMilkReactBindings = (
  react,
  {
    createEngine = createRustyMilkEngine,
    packLoader = loadRustyMilkPack,
  } = {},
) => {
  const React = resolveReact(react);
  const {
    useCallback,
    useEffect,
    useMemo,
    useRef,
    useState,
  } = React;

  const useRustyMilkEngine = ({
    audioContext,
    audioNode,
    canvas,
    modulePath,
    rendererBackend = 'webgl',
    presetText,
    presetFileName = 'rustymilk-react.milk',
    textureAssets = {},
    automation,
    onFrame,
    onError,
    frameLimitFps = 0,
    autoStart = true,
    ...engineConfig
  } = {}) => {
    const engineRef = useRef(null);
    const loopHandleRef = useRef(0);
    const mountedRef = useRef(true);
    const [status, setStatus] = useState('idle');
    const [error, setError] = useState('');
    const targetFps = useRef(clampFps(frameLimitFps));
    const lastFrameMs = useRef(0);
    const textureAssetsRef = useRef(textureAssets || {});

    const stopLoop = useCallback(() => {
      if (loopHandleRef.current) {
        const scheduler = getFrameScheduler();
        scheduler.cancelFrame(loopHandleRef.current);
        loopHandleRef.current = 0;
      }
    }, []);

    const dispose = useCallback(() => {
      stopLoop();
      const engine = engineRef.current;
      engineRef.current = null;
      engine?.dispose?.();
    }, [stopLoop]);

    const step = useCallback(() => {
      const engine = engineRef.current;
      if (!engine) return;
      const scheduler = getFrameScheduler();
      try {
        const now = globalThis.performance ? globalThis.performance.now() : Date.now();
        const targetMs = targetFps.current > 0 ? (1000 / targetFps.current) : 0;
        if (targetMs > 0 && lastFrameMs.current && now - lastFrameMs.current < targetMs) {
          loopHandleRef.current = scheduler.requestFrame(() => {
            loopHandleRef.current = 0;
            step();
          });
          return;
        }
        lastFrameMs.current = now;
        const update = engine.render();
        onFrame?.(update);
      } catch (nextError) {
        setError(String(nextError?.message || nextError || 'unknown render error'));
        onError?.(nextError);
      }
      loopHandleRef.current = scheduler.requestFrame(() => {
        loopHandleRef.current = 0;
        step();
      });
    }, [onError, onFrame]);

    const start = useCallback(async () => {
      if (!audioContext || !audioNode || !canvas) {
        return;
      }
      setStatus('initializing');
      setError('');
      try {
        const engine = await createEngine({
          audioContext,
          audioNode,
          canvas,
          modulePath,
          rendererBackend,
          ...engineConfig,
        });
        if (!mountedRef.current) {
          engine?.dispose?.();
          return;
        }
        engineRef.current = engine;
        if (automation) {
          engine.setPresetAutomation(automation);
        }
        if (presetText) {
          engine.loadPresetText(presetText, presetFileName);
        }
        setStatus('ready');
        step();
      } catch (nextError) {
        setError(String(nextError?.message || nextError || 'unknown engine error'));
        setStatus('error');
        onError?.(nextError);
      }
    }, [audioContext, audioNode, canvas, modulePath, rendererBackend, automation, onError, presetFileName, presetText, engineConfig, step]);

    useEffect(() => {
      targetFps.current = clampFps(frameLimitFps);
      textureAssetsRef.current = textureAssets || {};
      return () => {
        mountedRef.current = false;
        dispose();
      };
    }, [frameLimitFps, textureAssets, dispose]);

    useEffect(() => {
      if (!autoStart) {
        return undefined;
      }
      mountedRef.current = true;
      start();
      return () => {
        mountedRef.current = false;
        dispose();
      };
    }, [autoStart, start, dispose]);

    return useMemo(() => ({
      status,
      error,
      engine: engineRef.current,
      start,
      dispose,
      render: () => engineRef.current?.render?.(),
      getPresetDebugSnapshot: () => engineRef.current?.getPresetDebugSnapshot?.() || null,
      getPresetParameterSummary: () => engineRef.current?.getPresetParameterSummary?.() || null,
      getPresetFragmentSummary: () => engineRef.current?.getPresetFragmentSummary?.() || null,
      loadPresetText: (source, fileName, options = {}) =>
        engineRef.current?.loadPresetText?.(source, fileName, {
          ...options,
          textureAssets: textureAssetsRef.current,
        }),
      loadPresetFragmentText: (source, fileName, options = {}) =>
        engineRef.current?.loadPresetFragmentText?.(source, fileName, {
          ...options,
          textureAssets: textureAssetsRef.current,
        }),
      updatePresetBaseValue: (key, value, options = {}) =>
        engineRef.current?.updatePresetBaseValue?.(key, value, {
          ...options,
          textureAssets: textureAssetsRef.current,
        }),
      randomizePresetParameters: (options = {}) =>
        engineRef.current?.randomizePresetParameters?.({
          ...options,
          textureAssets: textureAssetsRef.current,
        }),
      removePresetFragment: (type, index, options = {}) =>
        engineRef.current?.removePresetFragment?.(type, index, {
          ...options,
          textureAssets: textureAssetsRef.current,
        }),
      inspectPresetText: (source, fileName) => engineRef.current?.inspectPresetText?.(source, fileName),
      exportPresetText: () => engineRef.current?.exportPresetText?.(),
      exportPresetFragment: (type, index) => engineRef.current?.exportPresetFragment?.(type, index),
      setMouseState: (mouseState) => engineRef.current?.setMouseState?.(mouseState),
      setPresetAutomation: (nextAutomation) => engineRef.current?.setPresetAutomation?.(nextAutomation),
      setTextureAssets: (nextTextureAssets) => {
        textureAssetsRef.current = nextTextureAssets || {};
      },
    }), [status, error, start, dispose]);
  };

  const useRustyMilkPack = (packUrl, { fetchImpl = globalThis.fetch } = {}) => {
    const [state, setState] = useState({
      status: 'idle',
      pack: null,
      error: '',
    });

    useEffect(() => {
      if (!packUrl) {
        setState((current) => ({ ...current, status: 'idle', pack: null }));
        return undefined;
      }
      let active = true;
      const load = async () => {
        setState({ status: 'loading', pack: null, error: '' });
        try {
          const loaded = await packLoader(packUrl, { fetchImpl });
          if (!active) return;
          setState({ status: 'ready', pack: loaded, error: '' });
        } catch (nextError) {
          if (!active) return;
          setState({
            status: 'error',
            pack: null,
            error: String(nextError?.message || nextError || 'failed to load pack'),
          });
        }
      };
      load();
      return () => {
        active = false;
      };
    }, [packUrl, fetchImpl]);

    return state;
  };

  const defaultCanvasProps = {
    role: 'img',
    'aria-label': 'RustyMilk visualizer canvas',
  };
  const RustyMilkCanvas = React.forwardRef ? React.forwardRef((props, ref) => {
    const style = props?.style || {};
    return React.createElement('canvas', {
      ...defaultCanvasProps,
      ...props,
      style,
      ref,
    });
  }) : ((props) => React.createElement('canvas', {
    ...defaultCanvasProps,
    ...props,
  }));

  return {
    useRustyMilkEngine,
    useRustyMilkPack,
    RustyMilkCanvas,
  };
};

export const createRustyMilkReact = createRustyMilkReactBindings;
