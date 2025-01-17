use gosub_render_backend::layout::{LayoutTree, Layouter};
use gosub_render_backend::{NodeDesc, RenderBackend, WindowedEventLoop};
use gosub_renderer::draw::SceneDrawer;
use gosub_shared::traits::css3::CssSystem;
use gosub_shared::traits::document::Document;
use gosub_shared::traits::html5::Html5Parser;
use gosub_shared::traits::render_tree::RenderTree;
use gosub_shared::types::Result;
use log::info;
use slotmap::{DefaultKey, SlotMap};
use std::sync::mpsc::Sender;
use url::Url;

pub struct Tabs<
    D: SceneDrawer<B, L, LT, Doc, C, RT>,
    B: RenderBackend,
    L: Layouter,
    LT: LayoutTree<L>,
    Doc: Document<C>,
    C: CssSystem,
    RT: RenderTree<C>,
> {
    #[allow(clippy::type_complexity)]
    pub tabs: SlotMap<DefaultKey, Tab<D, B, L, LT, Doc, C, RT>>,
    pub active: TabID,
    _marker: std::marker::PhantomData<(B, L, LT)>,
}

impl<
        D: SceneDrawer<B, L, LT, Doc, C, RT>,
        L: Layouter,
        LT: LayoutTree<L>,
        B: RenderBackend,
        Doc: Document<C>,
        C: CssSystem,
        RT: RenderTree<C>,
    > Default for Tabs<D, B, L, LT, Doc, C, RT>
{
    fn default() -> Self {
        Self {
            tabs: SlotMap::new(),
            active: TabID::default(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<
        D: SceneDrawer<B, L, LT, Doc, C, RT>,
        L: Layouter,
        LT: LayoutTree<L>,
        B: RenderBackend,
        Doc: Document<C>,
        C: CssSystem,
        RT: RenderTree<C>,
    > Tabs<D, B, L, LT, Doc, C, RT>
{
    pub fn new(initial: Tab<D, B, L, LT, Doc, C, RT>) -> Self {
        let mut tabs = SlotMap::new();
        let active = TabID(tabs.insert(initial));

        Self {
            tabs,
            active,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_tab(&mut self, tab: Tab<D, B, L, LT, Doc, C, RT>) -> TabID {
        TabID(self.tabs.insert(tab))
    }

    pub fn remove_tab(&mut self, id: TabID) {
        self.tabs.remove(id.0);
    }

    pub fn activate_tab(&mut self, id: TabID) {
        self.active = id;
    }

    pub fn get_current_tab(&mut self) -> Option<&mut Tab<D, B, L, LT, Doc, C, RT>> {
        self.tabs.get_mut(self.active.0)
    }

    #[allow(unused)]
    pub(crate) async fn from_url<P: Html5Parser<C, Document = Doc>>(
        url: Url,
        layouter: L,
        debug: bool,
    ) -> Result<Self> {
        let tab = Tab::from_url::<P>(url, layouter, debug).await?;
        Ok(Self::new(tab))
    }

    pub fn select_element(&mut self, id: LT::NodeId) {
        if let Some(tab) = self.get_current_tab() {
            tab.data.select_element(id);
        }
    }

    pub fn send_nodes(&mut self, sender: Sender<NodeDesc>) {
        if let Some(tab) = self.get_current_tab() {
            tab.data.send_nodes(sender);
        }
    }

    pub fn unselect_element(&mut self) {
        if let Some(tab) = self.get_current_tab() {
            tab.data.unselect_element();
        }
    }
}

#[derive(Debug)]
pub struct Tab<
    D: SceneDrawer<B, L, LT, Doc, C, RT>,
    B: RenderBackend,
    L: Layouter,
    LT: LayoutTree<L>,
    Doc: Document<C>,
    C: CssSystem,
    RT: RenderTree<C>,
> {
    pub title: String,
    pub url: Url,
    pub data: D,
    #[allow(clippy::type_complexity)]
    _marker: std::marker::PhantomData<fn(B, L, LT, Doc, C, RT)>,
}

impl<
        D: SceneDrawer<B, L, LT, Doc, C, RT>,
        B: RenderBackend,
        L: Layouter,
        LT: LayoutTree<L>,
        Doc: Document<C>,
        C: CssSystem,
        RT: RenderTree<C>,
    > Tab<D, B, L, LT, Doc, C, RT>
{
    pub fn new(title: String, url: Url, data: D) -> Self {
        Self {
            title,
            url,
            data,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn from_url<P: Html5Parser<C, Document = Doc>>(url: Url, layouter: L, debug: bool) -> Result<Self> {
        let data = D::from_url::<P>(url.clone(), layouter, debug).await?;

        info!("Tab created: {}", url.as_str());

        Ok(Self {
            title: url.as_str().to_string(),
            url,
            data,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn reload<P: Html5Parser<C, Document = Doc>>(&mut self, el: impl WindowedEventLoop<B, RT, C>) {
        self.data.reload::<P>(el);
    }

    pub fn reload_from(&mut self, rt: RT) {
        self.data.reload_from(rt)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TabID(pub(crate) DefaultKey);
