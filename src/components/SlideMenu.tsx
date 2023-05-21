import {ReactNode} from "react";
import {Button} from "primereact/button";
import {useLocation} from "react-router";

interface ComponentProps {
    subtitle: string | undefined
    onSubtitleClicked: () => void,
    onBackClicked: () => void
    children: ReactNode
}

export function SlideMenu(props: ComponentProps) {

    const location = useLocation();

    return (
        <div>
            {location?.pathname != '/' &&
                <div style={{display: 'flex', flexDirection: 'column'}}>
                    <div style={{display: 'flex', width: '100%', justifyContent: 'flex-start'}}>
                        <div style={{display: 'flex', alignItems: 'center', width: '100%'}}>
                            <Button outlined icon={"pi pi-arrow-left"}
                                    onClick={() => {
                                        props.onBackClicked()
                                    }}
                                    style={{
                                        height: '27px'
                                    }}/>
                            {props.subtitle && <Button onClick={props.onSubtitleClicked} text
                                                       style={{marginLeft: '10px'}}>{props.subtitle}</Button>}
                        </div>

                    </div>
                </div>
            }
            {props.children}
        </div>
    )
}
