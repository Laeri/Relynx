import { Button } from "primereact/button";
import { useRequestModelStore } from "../stores/requestStore";
import { SlideMenu } from "./SlideMenu";
import { CollectionMenuView } from "./collection/CollectionMenuView";
import { CollectionEntry } from "./collection/CollectionEntry";
import { Collection } from "../bindings";
import { Route, Routes, useLocation, useNavigate } from "react-router";
import { useState } from "react";

export enum NavbarSizes {
  Normal,
  Enlarged
}

export function Navbar() {

  const workspace = useRequestModelStore((state) => state.workspace);
  const currentCollection = useRequestModelStore((state) => state.currentCollection);

  const [navbarSize, setNavbarSize] = useState<NavbarSizes>(NavbarSizes.Normal);

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

  const enlargeNavbar = () => {
    setNavbarSize(NavbarSizes.Enlarged)
  }

  const shrinkNavbarToNormal = () => {
    setNavbarSize(NavbarSizes.Normal)
  }

  return (
    <nav style={{ padding: '30px 5px 10px 5px', position: 'relative', height: '100%', display: 'flex', flexDirection: 'column' }} className={`${navbarSize == NavbarSizes.Normal ? 'navbar-normal' : ''} ${navbarSize == NavbarSizes.Enlarged ? 'navbar-enlarged' : ''}`} >
      <h1 style={{}}>Relynx</h1>

      {navbarSize == NavbarSizes.Normal && <Button onClick={enlargeNavbar} icon="pi pi-chevron-right" text size="small" style={{ position: 'absolute', right: '8px', top: '8px' }} />
      }

      {navbarSize == NavbarSizes.Enlarged && <Button onClick={shrinkNavbarToNormal} icon="pi pi-chevron-left" text size="small" style={{ position: 'absolute', right: '8px', top: '8px' }} />
      }

      <div className="nav-content" style={{ marginTop: '30px', flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
        <SlideMenu
          onSubtitleClicked={onSubtitleClicked}
          onBackClicked={onBackClicked}
          subtitle={currentCollection?.name}
        >
          <Routes>
            <Route path="" element={

              <div className={"fade-in-fast"} style={{ height: '100%', display: 'flex', flexDirection: 'column', flexGrow: '1' }}>

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
                  marginTop: '30px',
                  height: '100%',
                  flexGrow: 1
                }}>
                  {workspace.collections.map((collection: Collection, index: number) => {
                    return (
                      <CollectionEntry collection={collection}
                        key={index} />
                    )

                  })
                  }

                  {workspace.collections.length == 0 &&
                    <span>You haven't created any collections yet.</span>}
                </div>

              </div>} />
            <Route path="/collection/*" element={
              <>
                {currentCollection && <CollectionMenuView collection={currentCollection} />}
              </>
            } />
          </Routes>

        </SlideMenu>

      </div>
    </nav>
  )
}
