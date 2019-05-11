package com.github.apohrebniak.hoverui.ui;

import com.github.apohrebniak.hoverui.KvNodeService;
import com.github.apohrebniak.hoverui.domain.MemberEntity;
import com.vaadin.flow.component.Text;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;
import com.vaadin.flow.component.tabs.Tab;
import com.vaadin.flow.component.tabs.Tabs;
import com.vaadin.flow.router.Route;
import org.springframework.beans.factory.annotation.Autowired;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Route("node")
public class NodeView extends VerticalLayout {

  private static final String TAB_MEMBERS = "Members";
  private static final String TAB_KV = "Key-Values";

  private KvNodeService nodeService;
  private Map<Tab, TabPage> tabToPage = new HashMap<>();

  public NodeView(@Autowired KvNodeService nodeService) {
    this.nodeService = nodeService;
    setupUi();
  }

  private void setupUi() {
    add(new Text("Hover KV node on 127.0.0.1:1111"));

    /*Tabs*/
    Tabs tabs = new Tabs();
    tabs.setSizeFull();
    add(tabs);

    /*Tab pages*/
    TabPage membersPage = setupMembersPage();
    TabPage kvPage = setupKvPage();
    add(membersPage);
    add(kvPage);

    Tab tabMembers = new Tab(TAB_MEMBERS);
    Tab tabKv = new Tab(TAB_KV);
    tabToPage.put(tabMembers, membersPage);
    tabToPage.put(tabKv, kvPage);
    tabs.add(tabMembers);
    tabs.add(tabKv);
    tabs.setFlexGrowForEnclosedTabs(1);

    tabs.addSelectedChangeListener(event -> {
      Tab selectedTab = event.getSource().getSelectedTab();

      tabToPage.entrySet().stream()
          .forEach(e -> e.getValue().setVisible(false));

      tabToPage.get(selectedTab).setVisible(true);
    });

    tabs.setSelectedTab(tabMembers);
    membersPage.setVisible(true);
  }

  private TabPage setupMembersPage() {
    TabPage page = new TabPage();
    page.setVisible(false);

    List<MemberEntity> members = nodeService.getMembers();

    if (members != null) { //TODO react with error
      members.stream()
          .map(MemberListItem::from)
          .forEach(page::add);
    }

    return page;
  }

  private TabPage setupKvPage() {
    TabPage page = new TabPage();
    page.setVisible(false);
    return page;
  }
}
