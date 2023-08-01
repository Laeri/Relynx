import './App.css';
import { createContext, Ref, useEffect, useRef, useState } from 'react';
import 'primereact/resources/primereact.min.css';
import '../node_modules/primeicons/primeicons.css';
import './theme.css';
import { Workspace } from './bindings';
import { OverviewComponent } from './pages/OverviewComponent';
import { MemoryRouter, Routes, Route } from 'react-router';
import { Toast } from 'primereact/toast';
import { useRequestModelStore } from './stores/requestStore';
import { Navbar } from './components/Navbar';
import { ConfirmPopup } from 'primereact/confirmpopup';
import { Container as ModalContainer } from 'react-modal-promise';
import { RequestComponent } from './components/RequestComponent';
import { backend } from './rpc';
import { EnvironmentComponent } from './components/EnvironmentComponent';
import { CollectionOverviewComponent } from './components/collection/CollectionOverviewComponent';
import { Button } from 'primereact/button';
import { openErrorReportingModal } from './common/modal';
import { catchError } from './common/errorhandling';

export interface ToastContext {
  toast: Ref<any>,
  showSuccess: (title: string, detail: string, life?: number) => void
  showInfo: (title: string, detail: string, life?: number) => void
  showWarn: (title: string, detail: string, life?: number) => void
  showError: (title: string, detail: string) => void
  show: (params: { severity: string, summary: string, detail: string, life?: number }) => void
}

// we define the context below with an actual value
// @ts-ignore
export const ToastContext = createContext<ToastContext>();

export let ExternalToast: ToastContext | undefined;

export const routes = {
  root: "/",
  collection: "/collection",
  request: "/collection/request",
  environment: "/collection/environment",
}



function App() {

  const updateWorkspace = useRequestModelStore((state) => state.updateWorkspace);
  const setLogPath = useRequestModelStore((state) => state.setLogPath);

  const toastRef: any = useRef(null);

  const [initialized, setInitialized] = useState<boolean>(false);

  const toastContext = {
    toast: toastRef,
    showSuccess: (title: string, detail: string, life?: number) => {
      toastContext.show({ severity: 'success', summary: title, detail: detail, life: life });
    },
    showInfo: (title: string, detail: string, life?: number) => {
      toastContext.show({ severity: 'info', summary: title, detail: detail, life: life });
    },
    showWarn: (title: string, detail: string, life?: number) => {
      toastContext.show({ severity: 'warn', summary: title, detail: detail, life: life });
    },
    showError: (title: string, detail: string) => {
      if (!toastRef?.current) {
        console.error("No toast container present to show messages");
        return
      }
      toastRef?.current.show({
        severity: 'error',
        content: (<CustomErrorTemplate title={title} detail={detail} />),
        summary: title,
        detail: detail,
        life: 50000
      });
    },

    show: (params: { severity: string, summary: string, detail: string, life?: number }) => {
      if (!toastRef?.current) {
        console.error("No toast container present to show messages");
        return
      }
      toastRef?.current.show({
        severity: params.severity,
        summary: params.summary,
        detail: params.detail,
        life: params.life ?? 7000
      });
    }
  };


  // Load workspace initially from backend
  useEffect(() => {
    ExternalToast = toastContext;
    backend.loadWorkspace()
      .then((workspace: Workspace) => {
        updateWorkspace(workspace);
      }).catch(catchError);

    backend.get_log_path().then((logPath: string) => {
      setLogPath(logPath);
    });
  }, []);


  interface CustomErrorTemplateProps {
    title: string,
    detail: string
  }
  const CustomErrorTemplate = (props: CustomErrorTemplateProps) => {
    return (
      <div style={{ width: '100%' }}>
        <h3>{props.title}</h3>
        <p style={{marginTop: '20px'}}>{props.detail}</p>

        <Button severity='secondary' raised={true} style={{ marginTop: '20px' }} onClick={() => openErrorReportingModal(props. title, props.detail)}>Report Error</Button>
      </div>
    )
  }



  return (

    <div id="App">
      <ToastContext.Provider value={toastContext}>
        <MemoryRouter>
          <Navbar />

          <main style={{ padding: '10px 5%', display: 'relative' }}>
            <Routes>
              <Route element={<OverviewComponent />} path={routes.root} />
              <Route element={<CollectionOverviewComponent />} path={routes.collection} />
              <Route element={<RequestComponent key={1} />} path={routes.request} />
              <Route element={<EnvironmentComponent />} path={routes.environment} />
            </Routes>
          </main>
          {/*Container element for toast rendering: https://primereact.org/toast/*/}
          <Toast ref={toastRef} />

        </MemoryRouter>

        {/*Container element for confirm popups: https://primereact.org/confirmpopup/*/}
        <ConfirmPopup />
        {/*Container element for modals using react-modal-promise */}
        <ModalContainer />
      </ToastContext.Provider>
    </div>

  );
}

export default App;
