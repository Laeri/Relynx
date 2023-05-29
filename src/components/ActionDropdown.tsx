import {Button} from "primereact/button";
import {OverlayPanel} from "primereact/overlaypanel";
import React, {useRef} from "react";

interface ComponentProps {
    children: React.ReactNode,
    styles: any
}

export function ActionDropdown(props: ComponentProps) {

    const ref = useRef(null)

    const toggleOverlay = (e: any) => {
        // @ts-ignore
        ref.current.toggle(e)
    }


    return (
        <div>
            <Button icon={'pi pi-ellipsis-h'} className={' p-button-text'} style={{...(props.styles??{}), maxHeight: '10px'}}
                    onClick={toggleOverlay}/>

            <OverlayPanel ref={ref}>
                <div style={{display: 'flex', flexDirection: 'column'}}>
                    {props.children}
                </div>
            </OverlayPanel>
        </div>

    )
}