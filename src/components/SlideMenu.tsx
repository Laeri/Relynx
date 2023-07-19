import { ReactNode } from "react";
import { Button } from "primereact/button";
import { useLocation } from "react-router";
import { routes } from "../App";

interface ComponentProps {
  subtitle: string | undefined
  onSubtitleClicked: () => void,
  onBackClicked: () => void
  children: ReactNode
}

export function SlideMenu(props: ComponentProps) {

  const location = useLocation();

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', flexGrow: 1 }}>
      {location?.pathname != routes.root &&
        <div style={{ display: 'flex', flexDirection: 'column' }}>
          <div style={{ display: 'flex', width: '100%', justifyContent: 'flex-start' }}>
            <div style={{ display: 'flex', alignItems: 'center', width: '100%' }}>
              <Button outlined size="small" icon={(location?.pathname == routes.collection || location?.pathname == routes.request) ? "pi pi-home" : "pi pi-arrow-left"}
                onClick={() => {
                  props.onBackClicked()
                }}
                style={{
                  height: '25px',
                  width: '25px'
                }} />
              {props.subtitle && <Button onClick={props.onSubtitleClicked} text
                style={{ marginLeft: '10px' }}>{props.subtitle}</Button>}
            </div>

          </div>
        </div>
      }
      {props.children}
    </div>
  )
}
