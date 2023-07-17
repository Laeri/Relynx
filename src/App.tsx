import './App.css';
import { createContext, Ref, useContext, useEffect, useRef } from 'react';
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
import { catchError } from './common/errorhandling';

export interface ToastContext {
  toast: Ref<any>,
  showSuccess: (title: string, detail: string) => void
  showInfo: (title: string, detail: string, life?: number) => void
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

    // @TODO: REMOVE
    backend.isSignatureValid({ license_key: "", license_signature: "" });;
    backend.loadWorkspace()
      .then((workspace: Workspace) => {
        updateWorkspace(workspace);

        // checkLicenseAndContinue = () => {
        //   // @TODO:M3K-84 continue and check if license is valid, by decrypting it
        //   let valid = licenseValid(this.props.app.config.license_key)
        //   if (valid) {
        //     this.continueToOverview()
        //   } else {
        //     this.setState({
        //       errorText: t('login.error.license_unspecified')
        //     })
        //   }
        // }
      }).catch(catchError)
  }, []);




  const toastContext = {
    toast: toastRef,
    showSuccess: (title: string, detail: string) => {
      toastContext.show({ severity: 'success', summary: title, detail: detail, life: undefined });
    },
    showInfo: (title: string, detail: string, life?: number) => {
      toastContext.show({ severity: 'info', summary: title, detail: detail, life: life });
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

              <Route element={<RequestComponent key={1} />} path={"/collection/request"} />
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
