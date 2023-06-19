import { createContext, Ref, useContext, useEffect, useRef } from 'react';
import './App.css';
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
import { CollectionOverviewComponent } from './components/CollectionOverviewComponent';
import { backend } from './rpc';
import { EnvironmentComponent } from './components/EnvironmentComponent';

export interface ToastContext {
  toast: Ref<any>,
  showSuccess: (title: string, detail: string) => void
  showInfo: (title: string, detail: string) => void
  showWarn: (title: string, detail: string, life?: number) => void
  showError: (title: string, detail: string) => void
  show: (params: { severity: string, summary: string, detail: string, life?: number }) => void
}

// we define the context below with an actual value
// @ts-ignore
export const ToastContext = createContext<ToastContext>();

export let ExternalToast: ToastContext

function App() {

  const updateWorkspace = useRequestModelStore((state) => state.updateWorkspace);

  const toastRef: any = useRef(null);

  const toast = useContext(ToastContext);
  ExternalToast = toast;

  // Load workspace initially from backend
  useEffect(() => {
    backend.loadWorkspace()
      .then((workspace: Workspace) => {
        console.log('HERE: ', workspace)
        updateWorkspace(workspace);
      }).catch((err: any) => { console.log('CATCH ERR err', err) });
  }, []);


  const toastContext = {
    toast: toastRef,
    showSuccess: (title: string, detail: string) => {
      toastContext.show({ severity: 'success', summary: title, detail: detail, life: undefined });
    },
    showInfo: (title: string, detail: string) => {
      toastContext.show({ severity: 'info', summary: title, detail: detail, life: undefined });
    },
    showWarn: (title: string, detail: string, life?: number) => {
      toastContext.show({ severity: 'warn', summary: title, detail: detail, life: life });
    },
    showError: (title: string, detail: string) => {
      toastContext.show({ severity: 'error', summary: title, detail: detail, life: undefined });
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
    },
  }

  return (

    <div id="App">
      {/* <h1>Relynx</h1> */}
      <ToastContext.Provider value={toastContext}>
        <MemoryRouter>
          <Navbar />

          <main style={{ padding: '10px 5%', display: 'relative' }}>
            {/*          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'flex-end',
            marginTop: '10px'
          }}>
            {currentCollection !== undefined
              && <div style={{}}><RequestSearch /></div>
            }
          </div> */}

            <Routes>
              <Route element={<OverviewComponent />} path={"/"} />

              <Route element={<RequestComponent />} path={"/collection/request"} />
              <Route element={<CollectionOverviewComponent />} path={"/collection"} />
              <Route element={<EnvironmentComponent />} path={"/collection/environment"} />
            </Routes>
          </main>

        </MemoryRouter>

      </ToastContext.Provider>
      {/*Container element for toast rendering: https://primereact.org/toast/*/}
      <Toast ref={toastRef} />
      {/*Container element for confirm popups: https://primereact.org/confirmpopup/*/}
      <ConfirmPopup />

      <ModalContainer />
    </div>

  );
}

export default App;
