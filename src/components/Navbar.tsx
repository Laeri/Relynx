import {Button} from "primereact/button";
import {useRequestModelStore} from "../stores/requestStore";
import {SlideMenu} from "./SlideMenu";
import {CollectionMenuView} from "./CollectionMenuView";
import {CollectionEntry} from "./CollectionEntry";
// @TODO import {openCreateCollectionModal, openImportCollectionModal} from "../common/modal";
import {Workspace, Collection} from "../bindings";
import {Route, Routes, useLocation, useNavigate} from "react-router";

export function Navbar() {

    const workspace = useRequestModelStore((state) => state.workspace);
    const currentCollection = useRequestModelStore((state) => state.currentCollection);

    const navigate = useNavigate();
    const location = useLocation();

    const onSubtitleClicked = () => {
        // if we are already on the location do nothing
        if (location.pathname == '/collection') {
            return
        }
        navigate(-1);
    }

    const onBackClicked = () => {
        navigate(-1);
    }

    return (
        <nav style={{padding: '30px 5px 50px 5px', minWidth: '20%'}}>
            <h1 style={{}}>Relynx</h1>
            <div style={{marginTop: '30px', padding: '10px 30px'}}>
                <SlideMenu
                    onSubtitleClicked={onSubtitleClicked}
                    onBackClicked={onBackClicked}
                    subtitle={currentCollection?.name}
                >
                    <Routes>
                        <Route path="" element={

                            <div className={"fade-in-fast"}>
                                <h2>Collections</h2>
                                <div style={{
                                    display: 'flex',
                                    justifyContent: 'space-between',
                                    alignItems: 'center',
                                    flexWrap: "wrap"
                                }}>
                                </div>

                                <div style={{
                                    display: 'flex',
                                    flexDirection: 'column',
                                    alignItems: 'left',
                                    marginTop: '30px'
                                }}>
                                    {workspace.collections.map((collection: Collection, index: number) => {
                                        return (
                                            <CollectionEntry collection={collection}
                                                             key={index}/>
                                        )

                                    })
                                    }

                                    {workspace.collections.length == 0 &&
                                        <span>You haven't created any collections yet.</span>}
                                </div>

                            </div>}/>
                        <Route path="/collection/*" element={
                            <>
                                {currentCollection && <CollectionMenuView collection={currentCollection}/>}
                            </>
                        }/>
                    </Routes>

                </SlideMenu>

            </div>
        </nav>
    )
}
