package com.github.apohrebniak.hoverui.ui;

import com.github.apohrebniak.hoverui.KvNodeService;
import com.github.apohrebniak.hoverui.domain.MemberEntity;
import com.vaadin.flow.component.Text;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;
import com.vaadin.flow.component.tabs.Tab;
import com.vaadin.flow.component.tabs.Tabs;
import com.vaadin.flow.router.BeforeEvent;
import com.vaadin.flow.router.HasUrlParameter;
import com.vaadin.flow.router.Route;
import com.vaadin.flow.spring.annotation.UIScope;
import org.springframework.beans.factory.annotation.Autowired;

import java.net.URI;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

@Route("node")
@UIScope
public class NodeView extends VerticalLayout implements HasUrlParameter<String> {

  private static final String TAB_MEMBERS = "Members";
  private static final String TAB_KV = "Key-Values";

  private KvNodeService nodeService;
  private Map<Tab, TabPage> tabToPage = new HashMap<>();

  private String host;
  private Integer port;

  public NodeView(@Autowired KvNodeService nodeService) {
    this.nodeService = nodeService;
  }

  private void setupUi() {
    /*Title*/
    String title = String.format("KV node %s:%d", this.host, this.port);
    add(new Text(title));

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

    List<MemberEntity> members = nodeService
        .getMembers(URI.create("http://" + host + ":" + port + "/members"));

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

  @Override
  public void setParameter(BeforeEvent event, String parameter) {
    String[] args = parameter.split("_");
    if (args.length == 2) {
      this.host = args[0];
      this.port = Integer.valueOf(args[1]);
    }

    setupUi();
  }
}
